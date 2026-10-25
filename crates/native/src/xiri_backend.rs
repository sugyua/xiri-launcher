//! Xiri's backend. Network and disk work run away from egui's frame thread.
use crate::destinia_updates;
#[cfg(windows)]
use crate::http::Transfer;
use crate::http::{Http, Stop};
use crate::source::{GitHubSource, ReleaseSource};
use crate::xiri_repositories::{self, CatalogCache};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
#[cfg(windows)]
use xiri_launcher_core::protocol::JobPhase;
use xiri_launcher_core::protocol::{Backend, Capabilities, Command, DestiniaReleaseInfo, Event, LatestRelease, Origin, Source};
#[cfg(windows)]
use xiri_launcher_core::xiri::package_asset;
use xiri_launcher_core::xiri::{AppInfo, EntryKind, PackageEntry};
use xiri_launcher_core::{Arch, InstallSource, Installed, Os, Platform, Release};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalApp {
    app: AppInfo,
    installed: Installed,
    entry: PackageEntry,
    managed: bool,
}
struct Shared {
    events: Mutex<Vec<Event>>,
    catalog: Mutex<CatalogCache>,
    installed: Mutex<BTreeMap<String, LocalApp>>,
    releases: Mutex<BTreeMap<String, Release>>,
    jobs: Mutex<BTreeMap<String, Arc<Stop>>>,
    processes: Mutex<BTreeMap<String, std::process::Child>>,
    checking: AtomicBool,
    destinia_busy: AtomicBool,
    destinia_latest: Mutex<Option<DestiniaReleaseInfo>>,
    destinia_download: Mutex<Option<PathBuf>>,
    warning: Mutex<Option<String>>,
    wake: Box<dyn Fn() + Send + Sync>,
    root: PathBuf,
}
impl Shared {
    fn emit(&self, event: Event) {
        self.events.lock().unwrap_or_else(PoisonError::into_inner).push(event);
        (self.wake)();
    }
    fn commit_local(&self, id: &str, local: Option<LocalApp>) -> Result<(), String> {
        let mut installed = self.installed.lock().unwrap_or_else(PoisonError::into_inner);
        let mut next = installed.clone();
        match local {
            Some(local) => {
                next.insert(id.into(), local);
            }
            None => {
                next.remove(id);
            }
        }
        let bytes = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
        crate::paths::write_atomic(&self.root.join("installed-xiri.json"), &bytes)?;
        *installed = next;
        Ok(())
    }
    fn app(&self, id: &str) -> Option<AppInfo> {
        self.catalog
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .apps
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .or_else(|| self.installed.lock().unwrap_or_else(PoisonError::into_inner).get(id).map(|a| a.app.clone()))
    }
    fn busy(&self, id: &str) -> bool {
        let mut processes = self.processes.lock().unwrap_or_else(PoisonError::into_inner);
        match processes.get_mut(id) {
            Some(child) => !matches!(child.try_wait(), Ok(Some(_))),
            None => false,
        }
    }
    fn detected(&self) {
        let mut apps = self.catalog.lock().unwrap_or_else(PoisonError::into_inner).apps.clone();
        let installed = self.installed.lock().unwrap_or_else(PoisonError::into_inner);
        for local in installed.values() {
            if !apps.iter().any(|a| a.id == local.app.id) {
                apps.push(local.app.clone());
            }
        }
        for app in apps {
            self.emit(Event::Detected {
                app: app.id.clone(),
                installed: installed.get(&app.id).filter(|l| Path::new(&l.installed.location).is_file()).map(|l| l.installed.clone()),
            });
        }
    }
    fn catalog_event(&self, warning: Option<String>) {
        if warning.is_some() {
            *self.warning.lock().unwrap_or_else(PoisonError::into_inner) = warning;
        }
        let warning = self.warning.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let mut apps = self.catalog.lock().unwrap_or_else(PoisonError::into_inner).apps.clone();
        for local in self.installed.lock().unwrap_or_else(PoisonError::into_inner).values() {
            if !apps.iter().any(|a| a.id == local.app.id) {
                apps.push(local.app.clone());
            }
        }
        self.emit(Event::CatalogLoaded { apps, warning });
        self.detected();
    }
}

pub struct XiriBackend {
    shared: Arc<Shared>,
}
fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> T {
    use std::io::Read;
    std::fs::File::open(path)
        .ok()
        .and_then(|file| {
            let mut bytes = Vec::new();
            file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes).ok()?;
            if bytes.len() > 16 * 1024 * 1024 {
                return None;
            }
            serde_json::from_slice(&bytes).ok()
        })
        .unwrap_or_default()
}
impl XiriBackend {
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        let root = dirs::data_local_dir().map(|p| p.join("Xiri Launcher"));
        let failure = root.is_none();
        let root = root.unwrap_or_default();
        let shared = Arc::new(Shared {
            events: Mutex::new(Vec::new()),
            catalog: Mutex::new(if failure { CatalogCache::default() } else { read_json(&root.join("catalog-xiri.json")) }),
            installed: Mutex::new(if failure { BTreeMap::new() } else { read_json(&root.join("installed-xiri.json")) }),
            releases: Mutex::new(BTreeMap::new()),
            jobs: Mutex::new(BTreeMap::new()),
            processes: Mutex::new(BTreeMap::new()),
            checking: AtomicBool::new(false),
            destinia_busy: AtomicBool::new(false),
            destinia_latest: Mutex::new(None),
            destinia_download: Mutex::new(None),
            warning: Mutex::new(None),
            wake: Box::new(wake),
            root,
        });
        {
            let mut cache = shared.catalog.lock().unwrap_or_else(PoisonError::into_inner);
            cache.apps.retain(AppInfo::trusted);
            cache.apps.truncate(2000);
        }
        shared.catalog_event(if failure { Some("Pasta de dados do Windows indisponível".into()) } else { None });
        Self { shared }
    }
    fn check(&self) {
        if self.shared.root.as_os_str().is_empty() {
            return;
        }
        if self.shared.checking.swap(true, Ordering::SeqCst) {
            return;
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            shared.emit(Event::CheckStarted);
            let answer = Http::new().and_then(|http| {
                let old = shared.catalog.lock().unwrap_or_else(PoisonError::into_inner).clone();
                match xiri_repositories::refresh(&http, &old) {
                    Ok(next) => {
                        let bytes = serde_json::to_vec(&next).map_err(|e| e.to_string())?;
                        crate::paths::write_atomic(&shared.root.join("catalog-xiri.json"), &bytes)?;
                        *shared.catalog.lock().unwrap_or_else(PoisonError::into_inner) = next;
                        *shared.warning.lock().unwrap_or_else(PoisonError::into_inner) = None;
                        shared.catalog_event(None);
                    }
                    Err(e) => shared.catalog_event(Some(e)),
                }
                let source = GitHubSource::new(http, shared.root.join("releases-xiri.json"));
                let apps = shared.catalog.lock().unwrap_or_else(PoisonError::into_inner).apps.clone();
                for app in apps {
                    let answer = source.latest(&app.repo, false);
                    if let Some(icon) = app.manifest.as_ref().and_then(|m| m.icon.as_ref())
                        && let Ok(Some(bytes)) = Http::new()
                            .and_then(|http| http.get_bytes(&format!("https://raw.githubusercontent.com/{}/HEAD/{icon}", app.repo), 512 * 1024))
                    {
                        shared.emit(Event::AppIcon { app: app.id.clone(), bytes });
                    }
                    if let LatestRelease::Found(r) = &answer.latest {
                        shared.releases.lock().unwrap_or_else(PoisonError::into_inner).insert(app.id.clone(), r.clone());
                    }
                    shared.emit(Event::Latest { app: app.id, latest: answer.latest, from: Origin::GitHub });
                }
                source.save();
                Ok(())
            });
            if let Err(e) = answer {
                shared.catalog_event(Some(e));
            }
            shared.emit(Event::CheckFinished { at: crate::now(), source: Source::GitHubApi, rate_limited_until: None });
            shared.checking.store(false, Ordering::SeqCst);
        });
    }
    fn check_destinia(&self) {
        if self.shared.destinia_busy.swap(true, Ordering::SeqCst) {
            return;
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let result = destinia_updates::check();
            *shared.destinia_latest.lock().unwrap_or_else(PoisonError::into_inner) = result.as_ref().ok().cloned();
            shared.emit(Event::DestiniaChecked(result));
            shared.destinia_busy.store(false, Ordering::SeqCst);
        });
    }
    fn download_destinia(&self) {
        if self.shared.destinia_busy.swap(true, Ordering::SeqCst) {
            return;
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let release = shared.destinia_latest.lock().unwrap_or_else(PoisonError::into_inner).clone();
            let result = match release {
                Some(release) => destinia_updates::download(&shared.root, &release),
                None => Err("Verifique primeiro a versao do Destinia".into()),
            };
            if let Ok(path) = &result {
                *shared.destinia_download.lock().unwrap_or_else(PoisonError::into_inner) = Some(path.clone());
            }
            shared.emit(Event::DestiniaDownloaded(result.map(|p| p.display().to_string())));
            shared.destinia_busy.store(false, Ordering::SeqCst);
        });
    }
    fn install_destinia(&self, folder: String, tag: String) {
        if self.shared.destinia_busy.swap(true, Ordering::SeqCst) {
            return;
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let release = shared.destinia_latest.lock().unwrap_or_else(PoisonError::into_inner).clone();
            let result = match release {
                Some(release) if release.tag == tag => destinia_updates::install(&shared.root, Path::new(&folder), &release),
                _ => Err("A versão do Destinia mudou. Verifique novamente antes de instalar.".into()),
            };
            shared.emit(Event::DestiniaInstalled(result));
            shared.destinia_busy.store(false, Ordering::SeqCst);
        });
    }
    fn reveal_destinia(&self) {
        let file = self.shared.destinia_download.lock().unwrap_or_else(PoisonError::into_inner).clone();
        #[cfg(windows)]
        if let Some(file) = file
            && let Some(folder) = file.parent()
        {
            let result = std::process::Command::new("explorer.exe").arg(folder).spawn();
            if result.is_err() {
                self.shared.emit(Event::DestiniaDownloaded(Err("Nao foi possivel abrir a pasta de downloads".into())));
            }
        }
        #[cfg(not(windows))]
        let _ = file;
    }
    fn run_job(&self, id: String, operation: impl FnOnce(&Arc<Shared>, &str, &Stop) -> Result<String, String> + Send + 'static) {
        if self.shared.root.as_os_str().is_empty() || !xiri_launcher_core::xiri::valid_entry_path(&id) || id.contains('/') {
            self.shared.emit(Event::JobFinished { app: id, result: Err("Pasta ou identificador inválido".into()) });
            return;
        }
        let stop = Arc::new(Stop::default());
        {
            let mut jobs = self.shared.jobs.lock().unwrap_or_else(PoisonError::into_inner);
            if jobs.contains_key(&id) {
                return;
            }
            jobs.insert(id.clone(), stop.clone());
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let result = operation(&shared, &id, &stop);
            shared.jobs.lock().unwrap_or_else(PoisonError::into_inner).remove(&id);
            shared.emit(Event::JobFinished { app: id, result });
            shared.catalog_event(None);
        });
    }
}

fn associate(shared: &Arc<Shared>, id: &str, path: &str) -> Result<String, String> {
    let app = shared.app(id).ok_or("Aplicativo desconhecido")?;
    if !app.trusted() {
        return Err("Repositório fora do perfil sugyua".into());
    }
    #[cfg(windows)]
    let target = dunce::canonicalize(path).map_err(|e| e.to_string())?;
    #[cfg(not(windows))]
    let target = Path::new(path).canonicalize().map_err(|e| e.to_string())?;
    if !target.is_file() {
        return Err("Selecione um arquivo EXE ou PS1".into());
    }
    let kind = match target.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("exe") => EntryKind::Exe,
        Some("ps1") => EntryKind::Powershell,
        _ => return Err("Selecione um arquivo EXE ou PS1".into()),
    };
    let entry = PackageEntry { kind, path: target.file_name().and_then(|s| s.to_str()).ok_or("Nome de arquivo inválido")?.into() };
    let installed = Installed { version: None, source: InstallSource::Binary, location: target.to_string_lossy().into(), package: None };
    shared.commit_local(id, Some(LocalApp { app, installed, entry, managed: false }))?;
    Ok("Programa local associado".into())
}

fn launch(shared: &Arc<Shared>, id: &str) -> Result<String, String> {
    if shared.busy(id) {
        return Err("Este programa já está em execução".into());
    }
    let local = shared.installed.lock().unwrap_or_else(PoisonError::into_inner).get(id).cloned().ok_or("Programa não instalado")?;
    let target = PathBuf::from(&local.installed.location);
    if !target.is_file() {
        return Err("Arquivo não encontrado; associe o programa novamente".into());
    }
    let mut command = match local.entry.kind {
        EntryKind::Exe => std::process::Command::new(&target),
        EntryKind::Powershell => {
            let system = std::env::var_os("SystemRoot").ok_or("Pasta Windows indisponível")?;
            let mut command = std::process::Command::new(PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe"));
            command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]).arg(&target);
            command
        }
    };
    if let Some(folder) = target.parent() {
        command.current_dir(folder);
    }
    let data = shared.root.join("apps").join(id).join("data");
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    command.env("XIRI_APP_DATA", &data);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let child = command.spawn().map_err(|e| format!("Não foi possível abrir o programa: {e}"))?;
    shared.processes.lock().unwrap_or_else(PoisonError::into_inner).insert(id.into(), child);
    Ok("Programa aberto".into())
}

#[cfg(windows)]
fn install(shared: &Arc<Shared>, id: &str, stop: &Stop) -> Result<String, String> {
    if shared.busy(id) {
        return Err("Feche o programa antes de atualizar".into());
    }
    let app = shared.app(id).ok_or("Aplicativo desconhecido")?;
    if !app.trusted() {
        return Err("Repositório fora do perfil sugyua".into());
    }
    let release = shared.releases.lock().unwrap_or_else(PoisonError::into_inner).get(id).cloned().ok_or("Sem release disponível")?;
    let asset = package_asset(&app, &release).ok_or("Pacote incompatível: requer manifesto, ZIP e SHA256SUMS.txt")?;
    if asset.size == 0 || asset.size > 2 * 1024 * 1024 * 1024 {
        return Err("Tamanho de pacote inválido".into());
    }
    if !xiri_launcher_core::release::download_url_under(&asset.url, &xiri_launcher_core::release::download_prefix(&app.repo)) {
        return Err("Origem do pacote não autorizada".into());
    }
    let http = Http::xiri()?;
    let sums_asset = release.assets.iter().find(|a| a.name == "SHA256SUMS.txt").ok_or("Checksum não publicado")?;
    let sums = http.get_text(&sums_asset.url)?.ok_or("Arquivo de checksums ausente")?;
    let hash = xiri_launcher_core::release::parse_sha256sums(&sums)
        .into_iter()
        .find(|(_, name)| name == &asset.name)
        .map(|(hash, _)| hash)
        .ok_or("Pacote não consta no SHA256SUMS.txt")?;
    let download = shared.root.join("downloads").join(format!("{id}-{hash}.zip"));
    shared.emit(Event::Progress { app: id.into(), phase: JobPhase::Downloading { done: 0, total: asset.size } });
    let pinned_url = http.pinned_release_url(&app.repo, &asset.url)?;
    let download_result = http.download(&pinned_url, &download, asset.size, &hash, stop, &mut |progress| {
        if let Transfer::Bytes { done, total } = progress {
            shared.emit(Event::Progress { app: id.into(), phase: JobPhase::Downloading { done, total } });
        }
    });
    if download_result.as_ref().is_err_and(|e| e == xiri_launcher_core::protocol::PAUSED) {
        shared.emit(Event::Paused {
            app: id.into(),
            download: Some(xiri_launcher_core::protocol::PausedDownload {
                done: crate::http::partial_bytes(&download, asset.size).unwrap_or(0),
                total: asset.size,
            }),
        });
    }
    download_result?;
    shared.emit(Event::Paused { app: id.into(), download: None });
    stop.check()?;
    shared.emit(Event::Progress { app: id.into(), phase: JobPhase::Verifying });
    let app_root = shared.root.join("apps").join(id);
    std::fs::create_dir_all(&app_root).map_err(|e| e.to_string())?;
    let staging = tempfile::Builder::new().prefix("staging-").tempdir_in(&app_root).map_err(|e| e.to_string())?;
    let file = std::fs::File::open(&download).map_err(|e| e.to_string())?;
    crate::xiri_package::extract_verified(file, staging.path())?;
    let manifest = app.manifest.as_ref().ok_or("Manifesto ausente")?;
    crate::xiri_package::validate_package(staging.path(), manifest)?;
    stop.check()?;
    if shared.busy(id) {
        return Err("Feche o programa antes de atualizar".into());
    }
    shared.emit(Event::Progress { app: id.into(), phase: JobPhase::Installing });
    let destination = app_root.join(&hash);
    if !destination.exists() {
        std::fs::rename(staging.path(), &destination).map_err(|e| e.to_string())?;
    }
    let target = crate::xiri_package::validate_package(&destination, manifest)?;
    let entry = manifest.windows.as_ref().ok_or("Sem entrada Windows")?.entry.clone();
    let installed = Installed {
        version: release.version.clone(),
        source: InstallSource::Launcher,
        location: target.to_string_lossy().into(),
        package: Some(hash),
    };
    shared.commit_local(id, Some(LocalApp { app, installed, entry, managed: true }))?;
    Ok("Programa instalado. Dados e versões anteriores foram preservados.".into())
}
#[cfg(not(windows))]
fn install(_: &Arc<Shared>, _: &str, _: &Stop) -> Result<String, String> {
    Err("Instalação Xiri disponível inicialmente no Windows x64".into())
}

impl Backend for XiriBackend {
    fn dynamic_catalog(&self) -> bool {
        true
    }
    fn catalog(&self) -> Vec<AppInfo> {
        self.shared.catalog.lock().unwrap_or_else(PoisonError::into_inner).apps.clone()
    }
    fn send(&mut self, command: Command) {
        match command {
            Command::CheckForUpdates { .. } => self.check(),
            Command::DestiniaCheck => self.check_destinia(),
            Command::DestiniaDownload => self.download_destinia(),
            Command::DestiniaInstall { folder, tag } => self.install_destinia(folder, tag),
            Command::DestiniaReveal => self.reveal_destinia(),
            Command::Detect => self.shared.catalog_event(None),
            Command::Install { app } | Command::Replace { app } => self.run_job(app, install),
            Command::Launch { app } => self.run_job(app, |shared, id, _| launch(shared, id)),
            Command::Associate { app, path } => self.run_job(app, move |shared, id, _| associate(shared, id, &path)),
            Command::Uninstall { app } => self.run_job(app, |shared, id, _| {
                if shared.busy(id) {
                    return Err("Feche o programa antes de removê-lo".into());
                }
                shared.commit_local(id, None)?;
                Ok("Associação removida; arquivos e dados foram preservados".into())
            }),
            Command::Reveal { app } => self.run_job(app, |shared, id, _| {
                let local = shared.installed.lock().unwrap_or_else(PoisonError::into_inner).get(id).cloned().ok_or("Não instalado")?;
                #[cfg(windows)]
                std::process::Command::new("explorer.exe")
                    .arg(Path::new(&local.installed.location).parent().ok_or("Pasta inválida")?)
                    .spawn()
                    .map_err(|e| e.to_string())?;
                #[cfg(not(windows))]
                let _ = local;
                Ok("Pasta aberta".into())
            }),
            Command::Pause { app } => {
                if let Some(stop) = self.shared.jobs.lock().unwrap_or_else(PoisonError::into_inner).get(&app) {
                    stop.pause();
                }
            }
            Command::Cancel { app } => {
                if let Some(stop) = self.shared.jobs.lock().unwrap_or_else(PoisonError::into_inner).get(&app) {
                    stop.cancel();
                }
                self.shared.emit(Event::Paused { app, download: None });
            }
            Command::UpdateLauncher => {}
            Command::Configure(_) | Command::FetchScreenshots { .. } => {}
        }
    }
    fn poll(&mut self) -> Vec<Event> {
        std::mem::take(&mut *self.shared.events.lock().unwrap_or_else(PoisonError::into_inner))
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities { manage_apps: cfg!(all(windows, target_arch = "x86_64")), reveal_label: "Show in file manager", self_update: false }
    }
    fn platform(&self) -> Option<Platform> {
        Some(Platform { os: Os::Windows, arch: Arch::X64 })
    }
    fn now(&self) -> i64 {
        crate::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn shared(root: &Path) -> Arc<Shared> {
        Arc::new(Shared {
            events: Mutex::new(Vec::new()),
            catalog: Mutex::new(CatalogCache::default()),
            installed: Mutex::new(BTreeMap::new()),
            releases: Mutex::new(BTreeMap::new()),
            jobs: Mutex::new(BTreeMap::new()),
            processes: Mutex::new(BTreeMap::new()),
            checking: AtomicBool::new(false),
            destinia_busy: AtomicBool::new(false),
            destinia_latest: Mutex::new(None),
            destinia_download: Mutex::new(None),
            warning: Mutex::new(None),
            wake: Box::new(|| {}),
            root: root.into(),
        })
    }
    fn local(path: &Path) -> LocalApp {
        let mut app = AppInfo::from(&xiri_launcher_core::SUITE[0]);
        app.id = "sensor-panel".into();
        app.repo = "sugyua/sensor-panel".into();
        LocalApp {
            app,
            installed: Installed { version: None, source: InstallSource::Binary, location: path.to_string_lossy().into(), package: None },
            entry: PackageEntry { kind: EntryKind::Exe, path: "app.exe".into() },
            managed: false,
        }
    }
    #[test]
    fn failed_state_write_preserves_the_previous_installation() {
        let dir = tempfile::tempdir().unwrap();
        let blocked = dir.path().join("file");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let shared = shared(&blocked);
        let old = local(&blocked);
        shared.installed.lock().unwrap().insert("sensor-panel".into(), old);
        assert!(shared.commit_local("sensor-panel", None).is_err());
        assert!(shared.installed.lock().unwrap().contains_key("sensor-panel"));
    }
    #[test]
    fn missing_remote_repository_keeps_local_launch_metadata_and_offline_warning() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("app.exe");
        std::fs::write(&target, b"fixture").unwrap();
        let shared = shared(dir.path());
        shared.commit_local("sensor-panel", Some(local(&target))).unwrap();
        shared.catalog_event(Some("offline".into()));
        shared.catalog_event(None);
        let events = shared.events.lock().unwrap();
        assert!(
            events.iter().any(|event| matches!(event,Event::CatalogLoaded {apps,warning} if apps.len()==1 && warning.as_deref()==Some("offline")))
        );
        assert!(events.iter().any(|event| matches!(event,Event::Detected {app,installed:Some(_)} if app=="sensor-panel")));
        assert!(target.exists());
    }
}
