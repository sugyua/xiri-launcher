//! Safe updates for the public Destinia Windows Releases.
//! Download from fixed GitHub URLs, check the published SHA-256, and overlay
//! only vetted package files after the user selects a valid installation.
use crate::http::{Fetched, Http, Stop};
use serde::Deserialize;
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
#[cfg(windows)]
use std::io::Write;
use std::path::{Path, PathBuf};
use xiri_launcher_core::protocol::DestiniaReleaseInfo;

const REPO: &str = "sugyua/destinia";
const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
#[cfg(windows)]
const MAX_UNPACKED: u64 = 1024 * 1024 * 1024;

#[derive(Deserialize)]
struct Asset {
    name: String,
    #[serde(default)]
    size: u64,
}
#[derive(Deserialize)]
struct Entry {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<Asset>,
}

fn asset_name(tag: &str) -> Option<String> {
    let suffix = tag.strip_prefix('v')?;
    if suffix.len() < 3
        || suffix.len() > 64
        || !suffix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        || !suffix.as_bytes()[0].is_ascii_digit()
        || !suffix.contains('.')
    {
        return None;
    }
    Some(format!("destinia-{suffix}-windows-x64.zip"))
}

fn version_from_tag(tag: &str) -> Result<&str, String> {
    let suffix = tag.strip_prefix('v').ok_or("Tag Destinia inválida")?;
    let version = suffix.strip_suffix("-beta").unwrap_or(suffix);
    let parts = version.split('.').collect::<Vec<_>>();
    if (parts.len() != 2 && parts.len() != 3) || parts.iter().any(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit())) {
        return Err("Tag do Destinia não é uma versão numérica suportada".into());
    }
    Ok(version)
}

/// GitHub lists Releases newest first. Ignore source-only and incomplete releases.
pub fn parse_latest(json: &str) -> Result<DestiniaReleaseInfo, String> {
    let rows: Vec<Entry> = serde_json::from_str(json).map_err(|_| "Resposta inválida do GitHub")?;
    for row in rows {
        let Some(name) = asset_name(&row.tag_name) else { continue };
        if row.draft || version_from_tag(&row.tag_name).is_err() {
            continue;
        }
        let Some(zip) = row.assets.iter().find(|a| a.name == name) else { continue };
        if zip.size == 0 || zip.size > MAX_ARCHIVE {
            continue;
        }
        if row.assets.iter().any(|a| a.name == format!("{name}.sha256")) {
            return Ok(DestiniaReleaseInfo { tag: row.tag_name, asset: name, size: zip.size });
        }
    }
    Err("Nenhuma Release Windows completa foi encontrada no Destinia.".into())
}

pub fn check() -> Result<DestiniaReleaseInfo, String> {
    let response = Http::new()?.get_api("https://api.github.com/repos/sugyua/destinia/releases?per_page=30", None)?;
    match response {
        Fetched::Body { text, .. } => parse_latest(&text),
        Fetched::RateLimited { .. } => Err("GitHub atingiu o limite temporário de consultas. Tente novamente mais tarde.".into()),
        _ => Err("Não foi possível consultar as Releases públicas do Destinia.".into()),
    }
}

fn release_url(release: &DestiniaReleaseInfo, name: &str) -> Result<String, String> {
    if asset_name(&release.tag).as_deref() != Some(&release.asset) || release.size == 0 || release.size > MAX_ARCHIVE {
        return Err("Release inválida".into());
    }
    if name != release.asset && name != format!("{}.sha256", release.asset) {
        return Err("Arquivo não permitido".into());
    }
    Ok(format!("https://github.com/{REPO}/releases/download/{}/{name}", release.tag))
}

fn parse_checksum(text: &str, asset: &str) -> Result<String, String> {
    let mut fields = text.split_whitespace();
    let hash = fields.next().ok_or("SHA-256 ausente")?;
    let filename = fields.next().ok_or("Nome de arquivo ausente no checksum")?;
    if fields.next().is_some() || filename != asset || hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Arquivo SHA-256 inválido ou não corresponde ao pacote".into());
    }
    Ok(hash.to_ascii_lowercase())
}

fn digest(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0; 65536];
    let mut hasher = Sha256::new();
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn download(root: &Path, release: &DestiniaReleaseInfo) -> Result<PathBuf, String> {
    let zip_url = release_url(release, &release.asset)?;
    let hash_url = release_url(release, &format!("{}.sha256", release.asset))?;
    if root.as_os_str().is_empty() {
        return Err("Pasta de dados Xiri indisponível".into());
    }
    let http = Http::xiri()?;
    let checksum = http.get_text(&hash_url)?.ok_or("SHA-256 da Release ausente")?;
    let expected = parse_checksum(&checksum, &release.asset)?;
    let downloads = root.join("destinia-downloads").join(&release.tag);
    fs::create_dir_all(&downloads).map_err(|e| e.to_string())?;
    let destination = downloads.join(&release.asset);
    if destination.is_file() && digest(&destination)? == expected {
        inspect_archive(&destination, release)?;
        return Ok(destination);
    }
    // Existing bad files must not be reused; the normal downloader validates SHA256.
    let pinned = http.pinned_release_url(REPO, &zip_url)?;
    http.download(&pinned, &destination, release.size, &expected, &Stop::default(), &mut |_| {})?;
    inspect_archive(&destination, release)?;
    Ok(destination)
}

#[cfg(windows)]
fn package_files(zip_path: &Path, release: &DestiniaReleaseInfo) -> Result<Vec<PathBuf>, String> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path).map_err(|e| e.to_string())?).map_err(|e| format!("ZIP inválido: {e}"))?;
    if archive.is_empty() || archive.len() > 2000 {
        return Err("Quantidade de arquivos ZIP inválida".into());
    }
    let prefix = format!("destinia-{}/", release.tag.strip_prefix('v').ok_or("Tag inválida")?);
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name();
        if !name.starts_with(&prefix) {
            return Err("ZIP contém arquivos fora da pasta da Release".into());
        }
        let relative = name.strip_prefix(&prefix).ok_or("Prefixo ZIP inválido")?.trim_end_matches('/');
        if relative.is_empty() && file.is_dir() {
            continue;
        }
        if !safe_relative(relative) || file.is_symlink() {
            return Err("ZIP contém caminho perigoso".into());
        }
        if !seen.insert(relative.to_ascii_lowercase()) {
            return Err("ZIP com nomes duplicados".into());
        }
        if file.is_dir() {
            continue;
        }
        total = total.checked_add(file.size()).ok_or("Tamanho descompactado inválido")?;
        if total > MAX_UNPACKED || file.size() > file.compressed_size().max(1).saturating_mul(200) {
            return Err("ZIP excede os limites de extração".into());
        }
        names.push(PathBuf::from(relative));
    }
    for mandatory in ["VERSION", "Destinia.exe", "Destinia Zoom.exe", "host/build/destinia_host.exe", "host_zoom/build/destinia_host.exe"] {
        if !seen.contains(&mandatory.to_ascii_lowercase()) {
            return Err(format!("Arquivo obrigatório ausente no ZIP: {mandatory}"));
        }
    }
    let version_index =
        (0..archive.len()).find(|&i| archive.by_index(i).ok().is_some_and(|f| f.name() == format!("{prefix}VERSION"))).ok_or("VERSION ausente")?;
    let mut version = String::new();
    archive.by_index(version_index).map_err(|e| e.to_string())?.take(80).read_to_string(&mut version).map_err(|e| e.to_string())?;
    if version.trim() != version_from_tag(&release.tag)? {
        return Err("VERSION no pacote não corresponde à Release".into());
    }
    Ok(names)
}

#[cfg(windows)]
fn safe_relative(name: &str) -> bool {
    use xiri_launcher_core::xiri::valid_entry_path;
    if !valid_entry_path(name) {
        return false;
    }
    let low = name.to_ascii_lowercase();
    let first = low.split('/').next().unwrap_or("");
    !matches!(first, "game" | "save" | "saves" | "userdata" | "profiles" | ".git")
        && !first.starts_with(".xiri")
        && !low.ends_with(".dat")
        && !low.ends_with(".sav")
}

#[cfg(windows)]
fn inspect_archive(path: &Path, release: &DestiniaReleaseInfo) -> Result<(), String> {
    let _ = package_files(path, release)?;
    Ok(())
}
#[cfg(not(windows))]
fn inspect_archive(_: &Path, _: &DestiniaReleaseInfo) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn extract_verified(zip_path: &Path, staging: &Path, release: &DestiniaReleaseInfo) -> Result<Vec<PathBuf>, String> {
    let list = package_files(zip_path, release)?; // preflight before writing anything
    let mut archive = zip::ZipArchive::new(File::open(zip_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let prefix = format!("destinia-{}/", release.tag.strip_prefix('v').ok_or("Tag inválida")?);
    for relative in &list {
        let name = format!("{prefix}{}", relative.to_string_lossy().replace('\\', "/"));
        let mut input = archive.by_name(&name).map_err(|_| "Entrada do ZIP ausente")?;
        let output = staging.join(relative);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut target = File::create(&output).map_err(|e| e.to_string())?;
        let size = input.size();
        if std::io::copy(&mut input, &mut target).map_err(|e| e.to_string())? != size {
            return Err("Arquivo descompactado incompleto".into());
        }
        target.flush().map_err(|e| e.to_string())?;
    }
    Ok(list)
}

/// Reject symlinks/reparse points in the selected installation's file path.
#[cfg(windows)]
fn safe_target(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(m) => {
                if m.file_type().is_symlink() {
                    return Err("Link simbólico dentro da pasta de instalação".into());
                }
                let resolved = path.canonicalize().map_err(|e| e.to_string())?;
                if !resolved.starts_with(root) {
                    return Err("Arquivo redirecionado para fora do Destinia".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(path)
}

#[cfg(windows)]
fn apply_files(source: &Path, user_folder: &Path, files: &[PathBuf]) -> Result<String, String> {
    let root = user_folder.canonicalize().map_err(|_| "Pasta Destinia não existe")?;
    if !root.join("VERSION").is_file() || !root.join("game").is_dir() || !root.join("host").is_dir() || !root.join("host_zoom").is_dir() {
        return Err("Selecione a raiz de uma instalação existente do Destinia (VERSION, game, host e host_zoom).".into());
    }
    if root.to_string_lossy().len() < 5 {
        return Err("Pasta de instalação insegura".into());
    }
    // Validate all destinations before starting an installation.
    for rel in files {
        let name = rel.to_string_lossy().replace('\\', "/");
        if !safe_relative(&name) {
            return Err("Arquivo protegido no pacote".into());
        }
        let target = safe_target(&root, rel)?;
        if target.exists() && !target.is_file() {
            return Err(format!("Destino não é arquivo: {}", rel.display()));
        }
    }
    // Backup is created on the SAME drive, for rename and exact rollback.
    let backup = tempfile::Builder::new().prefix(".xiri-destinia-backup-").tempdir_in(&root).map_err(|e| e.to_string())?;
    let mut applied: Vec<(PathBuf, bool)> = Vec::new();
    let attempt: Result<(), String> = (|| {
        for rel in files {
            let target = safe_target(&root, rel)?;
            let from = source.join(rel);
            let parent = target.parent().ok_or("Pasta de destino inválida")?;
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            let mut staging_file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
            let mut source_file = File::open(&from).map_err(|e| format!("Falha ao abrir arquivo: {e}"))?;
            std::io::copy(&mut source_file, staging_file.as_file_mut()).map_err(|e| format!("Falha ao preparar arquivo: {e}"))?;
            staging_file.as_file_mut().sync_all().map_err(|e| e.to_string())?;
            let had_previous = target.is_file();
            if had_previous {
                let old = backup.path().join(rel);
                if let Some(folder) = old.parent() {
                    fs::create_dir_all(folder).map_err(|e| e.to_string())?;
                }
                fs::rename(&target, &old).map_err(|_| format!("Feche o Destinia antes da atualização: {}", rel.display()))?;
            }
            applied.push((rel.clone(), had_previous));
            staging_file.persist(&target).map_err(|e| format!("Falha ao substituir {}: {}", rel.display(), e.error))?;
        }
        Ok(())
    })();
    if let Err(error) = attempt {
        let mut problems = Vec::new();
        for (rel, existed) in applied.iter().rev() {
            let target = root.join(rel);
            if target.is_file()
                && let Err(e) = fs::remove_file(&target)
            {
                problems.push(format!("{}: {e}", rel.display()));
            }
            if *existed {
                let old = backup.path().join(rel);
                if let Err(e) = fs::rename(&old, &target) {
                    problems.push(format!("rollback {}: {e}", rel.display()));
                }
            }
        }
        if problems.is_empty() {
            return Err(format!("{error}. Alterações revertidas; seus dados originais continuam no lugar."));
        }
        let rescue = backup.keep();
        return Err(format!("{error}. Rollback incompleto: {}. Backups preservados em {}.", problems.join("; "), rescue.display()));
    }
    let saved = backup.keep();
    Ok(format!(
        "Destinia atualizado. {} arquivos do pacote aplicados; arquivos extras e saves preservados. Backup em {}.",
        files.len(),
        saved.display()
    ))
}

#[cfg(windows)]
pub fn install(data_root: &Path, folder: &Path, release: &DestiniaReleaseInfo) -> Result<String, String> {
    if fs::read_to_string(folder.join("VERSION")).ok().is_some_and(|current| current.trim() == version_from_tag(&release.tag).unwrap_or("")) {
        return Ok(format!("Destinia {} já está instalado nesta pasta; nada foi modificado.", release.tag));
    }
    // Download and stage EVERYTHING before modifying the installation.
    let zip_file = download(data_root, release)?;
    let work = tempfile::tempdir_in(data_root).map_err(|e| e.to_string())?;
    let files = extract_verified(&zip_file, work.path(), release)?;
    apply_files(work.path(), folder, &files)
}
#[cfg(not(windows))]
pub fn install(_: &Path, _: &Path, _: &DestiniaReleaseInfo) -> Result<String, String> {
    Err("O instalador do Destinia requer Windows x64".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_public_windows_release_and_ignore_incomplete() {
        let json = r#"[{"tag_name":"v0.9.3-beta","assets":[{"name":"source.zip","size":50}]},{"tag_name":"v0.9.2-beta","assets":[{"name":"destinia-0.9.2-beta-windows-x64.zip","size":1024},{"name":"destinia-0.9.2-beta-windows-x64.zip.sha256","size":100}]}]"#;
        let r = parse_latest(json).unwrap();
        assert_eq!(r.tag, "v0.9.2-beta");
        assert_eq!(r.size, 1024);
        assert!(parse_latest(r#"[{"tag_name":"v../evil-beta","assets":[]}]"#).is_err());
        assert!(parse_latest(r#"[{"tag_name":"v0.9.2-beta","draft":true,"assets":[{"name":"destinia-0.9.2-beta-windows-x64.zip","size":1024},{"name":"destinia-0.9.2-beta-windows-x64.zip.sha256"}]}]"#).is_err());
        assert!(parse_checksum(&format!("{}  destinia.zip\n", "a".repeat(64)), "other.zip").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn overlays_only_package_files_and_preserves_saves_and_custom_data() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("Destinia");
        fs::create_dir_all(game.join("game")).unwrap();
        fs::create_dir_all(game.join("host")).unwrap();
        fs::create_dir_all(game.join("host_zoom")).unwrap();
        fs::write(game.join("VERSION"), "0.9.1").unwrap();
        fs::write(game.join("host/old.exe"), "old").unwrap();
        fs::write(game.join("game/character.sav"), "valuable save").unwrap();
        fs::write(game.join("custom-not-in-package.cfg"), "my preferences").unwrap();
        let incoming = dir.path().join("stage");
        fs::create_dir_all(incoming.join("host")).unwrap();
        fs::write(incoming.join("VERSION"), "0.9.2").unwrap();
        fs::write(incoming.join("host/old.exe"), "updated").unwrap();
        let files = vec![PathBuf::from("VERSION"), PathBuf::from("host/old.exe")];
        let result = apply_files(&incoming, &game, &files).unwrap();
        assert!(result.contains("Backup"));
        assert_eq!(fs::read_to_string(game.join("VERSION")).unwrap(), "0.9.2");
        assert_eq!(fs::read_to_string(game.join("host/old.exe")).unwrap(), "updated");
        assert_eq!(fs::read_to_string(game.join("game/character.sav")).unwrap(), "valuable save");
        assert_eq!(fs::read_to_string(game.join("custom-not-in-package.cfg")).unwrap(), "my preferences");
        assert!(safe_relative("docs/SAVES_E_LOGS.md"));
        assert!(!safe_relative("game/character.sav"));
        assert!(!safe_relative("../escape.exe"));
    }

    #[cfg(windows)]
    #[test]
    fn failed_second_file_restores_all_overwritten_files() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("Destinia");
        for p in ["game", "host", "host_zoom"] {
            fs::create_dir_all(game.join(p)).unwrap();
        }
        fs::write(game.join("VERSION"), "0.9.1").unwrap();
        fs::write(game.join("game/savefile.sav"), "safeguarded").unwrap();
        let staged = dir.path().join("incoming");
        fs::create_dir(&staged).unwrap();
        fs::write(staged.join("VERSION"), "0.9.2").unwrap();
        let files = vec![PathBuf::from("VERSION"), PathBuf::from("missing-file.exe")];
        let error = apply_files(&staged, &game, &files).unwrap_err();
        assert!(error.contains("revertidas"));
        assert_eq!(fs::read_to_string(game.join("VERSION")).unwrap(), "0.9.1");
        assert_eq!(fs::read_to_string(game.join("game/savefile.sav")).unwrap(), "safeguarded");
        assert!(!game.join("missing-file.exe").exists());
    }

    #[cfg(windows)]
    #[test]
    fn malicious_or_game_overwriting_zip_is_rejected_before_install() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.zip");
        let release = DestiniaReleaseInfo { tag: "v0.9.2-beta".into(), asset: "destinia-0.9.2-beta-windows-x64.zip".into(), size: 1000 };
        let prefix = "destinia-0.9.2-beta/";
        {
            let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
            for file in [
                "VERSION",
                "Destinia.exe",
                "Destinia Zoom.exe",
                "host/build/destinia_host.exe",
                "host_zoom/build/destinia_host.exe",
                "game/secret.dat",
            ] {
                zip.start_file(format!("{prefix}{file}"), SimpleFileOptions::default()).unwrap();
                zip.write_all(if file == "VERSION" { b"0.9.2" } else { b"x" }).unwrap();
            }
            zip.finish().unwrap();
        }
        assert!(package_files(&path, &release).is_err());
    }
}
