//! Public GitHub Releases self-update for unsigned Xiri portable Beta builds.
//! The signed-only updater in self_update.rs serves signed installer variants;
//! this path pins the unsigned portable ZIP to SHA256SUMS.txt from our own repo.
//! A detached copy of this launcher waits until the original process exits.
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use sha2::{Digest, Sha256};
use xiri_launcher_core::release::{self, Release};
use xiri_launcher_core::protocol::JobPhase;
use xiri_launcher_core::version;
use crate::http::{Fetched, Http};
#[cfg(windows)]
use crate::http::{Stop, Transfer};

const REPO: &str = "sugyua/xiri-launcher";
#[cfg(any(windows, test))]
const FINISH: &str = "--xiri-apply-verified-update";
const MAX_ZIP: u64 = 256 * 1024 * 1024;

fn asset_name(tag: &str) -> Option<String> {
    let label = tag.strip_prefix('v')?;
    if label.len() > 65 || label.len() < 5
        || !label.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c,b'.' | b'-'))
        || !label.as_bytes()[0].is_ascii_digit()
    { return None; }
    Some(format!("xiri-launcher-{label}-windows-x64-portable.zip"))
}

/// Only return a newer, complete Windows portable Release published by our repo.
pub fn latest() -> Result<Option<Release>, String> {
    let response = Http::new()?.get_api("https://api.github.com/repos/sugyua/xiri-launcher/releases?per_page=30", None)?;
    let Fetched::Body { text, .. } = response else {
        return Err("Não foi possível consultar versões do Xiri (GitHub indisponível ou limite atingido)".into());
    };
    let Some(release) = release::parse_github_release_list(&text, REPO, true)? else { return Ok(None) };
    if !suitable(&release) { return Ok(None); }
    let Some(new) = release.version.as_ref() else { return Ok(None) };
    let Some(old) = version::parse(env!("CARGO_PKG_VERSION")) else { return Ok(None) };
    Ok(version::is_newer(new, &old).then_some(release))
}

fn suitable(release: &Release) -> bool {
    asset_name(&release.tag).is_some_and(|name| {
        release.assets.iter().any(|a| a.name == name && a.size > 0 && a.size <= MAX_ZIP)
            && release.assets.iter().any(|a| a.name == "SHA256SUMS.txt")
    })
}

#[cfg(windows)]
fn sha_file(path: &Path) -> Result<String, String> {
    let mut source = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = source.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 { break }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn checksum_for(body: &str, asset: &str) -> Result<String, String> {
    release::parse_sha256sums(body).into_iter()
        .find_map(|(sum, name)| (name == asset).then_some(sum))
        .ok_or("SHA256SUMS.txt não contém o ZIP Windows desta versão".into())
}

#[cfg(windows)]
fn archive_version(path: &Path, tag: &str) -> Result<(), String> {
    let expected = tag.strip_prefix('v').ok_or("Tag inválida")?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).map_err(|e| e.to_string())?)
        .map_err(|_| "ZIP do Xiri inválido")?;
    if archive.is_empty() || archive.len() > 1500 { return Err("Quantidade inválida de arquivos no ZIP do Xiri".into()); }
    for mandatory in ["VERSION", "xiri-launcher.exe"] {
        let mut entry = archive.by_name(mandatory).map_err(|_| format!("ZIP do Xiri não contém {mandatory}"))?;
        if mandatory == "VERSION" {
            if entry.size() > 80 { return Err("VERSION do ZIP inválido".into()); }
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(|_| "VERSION do ZIP ilegível")?;
            if content.trim().trim_start_matches('\u{feff}') != expected { return Err("O VERSION do ZIP não corresponde à Release".into()); }
        } else {
            if entry.size() < 4096 { return Err("xiri-launcher.exe do ZIP inválido".into()); }
            let mut magic = [0u8; 2];
            entry.read_exact(&mut magic).map_err(|e| e.to_string())?;
            if magic != *b"MZ" { return Err("O ZIP não contém um executável Windows válido".into()); }
        }
    }
    Ok(())
}

/// Preconditions for changing this portable installation. Never write into another app's folder.
#[cfg(windows)]
fn current_install() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?.canonicalize().map_err(|e| e.to_string())?;
    if exe.file_name().is_none_or(|n| !n.to_string_lossy().eq_ignore_ascii_case("xiri-launcher.exe")) {
        return Err("Atualização integrada disponível para o Xiri Launcher Windows portátil".into());
    }
    let folder = exe.parent().ok_or("Pasta do Xiri ausente")?;
    let previous = std::fs::read_to_string(folder.join("VERSION")).map_err(|_| "VERSION do Xiri não encontrado ao lado do executável")?;
    if previous.trim().trim_start_matches('\u{feff}') != env!("CARGO_PKG_VERSION") {
        return Err("Os arquivos do Xiri não correspondem à versão em execução; reinstale o ZIP completo".into());
    }
    Ok(exe)
}

#[cfg(windows)]
pub fn available() -> bool { current_install().is_ok() }
#[cfg(not(windows))]
pub fn available() -> bool { false }

/// Download everything before scheduling a helper. A SHA256 is verified again in the helper.
#[cfg(windows)]
pub fn schedule(release: &Release, data_root: &Path, mut progress: impl FnMut(JobPhase)) -> Result<(), String> {
    let target = current_install()?;
    if !suitable(release) { return Err("Release do Xiri sem ZIP Windows e checksums válidos".into()); }
    let version = release.version.as_ref().ok_or("Release sem versão")?;
    let installed = version::parse(env!("CARGO_PKG_VERSION")).ok_or("Versão instalada inválida")?;
    if !version::is_newer(version, &installed) { return Err("O Xiri já está atualizado".into()); }
    let name = asset_name(&release.tag).ok_or("Nome da Release inválido")?;
    let asset = release.assets.iter().find(|a| a.name == name).ok_or("ZIP Windows não encontrado")?;
    let sums_url = format!("https://github.com/{REPO}/releases/download/{}/SHA256SUMS.txt", release.tag);
    let http = Http::xiri()?;
    let expected = checksum_for(&http.get_text(&sums_url)?.ok_or("Checksums da Release indisponíveis")?, &name)?;
    if expected.len() != 64 || !expected.bytes().all(|c| c.is_ascii_hexdigit()) { return Err("SHA-256 inválido".into()); }
    let working = data_root.join("xiri-self-updates").join(&release.tag);
    std::fs::create_dir_all(&working).map_err(|e| e.to_string())?;
    let archive = working.join(&name);
    let pinned = http.pinned_release_url(REPO, &asset.url)?;
    http.download(&pinned, &archive, asset.size, &expected, &Stop::default(), &mut |status| {
        if let Transfer::Bytes { done, total } = status { progress(JobPhase::Downloading { done, total }); }
    })?;
    progress(JobPhase::Verifying);
    archive_version(&archive, &release.tag)?;
    // Freeze what was verified in the exact command given to our detached helper.
    let old_hash = sha_file(&target)?;
    let helper = working.join("xiri-launcher-updater.exe");
    std::fs::copy(&target, &helper).map_err(|e| e.to_string())?;
    if sha_file(&helper)? != old_hash { return Err("O auxiliar de atualização não corresponde ao Xiri atual".into()); }
    let helper_args = [FINISH.into(), std::process::id().to_string().into(), target.into_os_string(),
        archive.into_os_string(), expected.into(), old_hash.into(), release.tag.clone().into()];
    let mut cmd = std::process::Command::new(&helper);
    cmd.args(helper_args);
    crate::process::spawn_detached(cmd).map_err(|e| format!("Não foi possível preparar a atualização: {e}"))?;
    progress(JobPhase::Installing);
    Ok(())
}
#[cfg(not(windows))]
pub fn schedule(_: &Release, _: &Path, _: impl FnMut(JobPhase)) -> Result<(), String> {
    Err("Atualização automática do Xiri disponível somente no Windows".into())
}

/// Child process: trust only the exact ZIP whose checksum was validated before exit.
#[cfg(windows)]
pub fn finish(args: &[OsString]) -> Option<Result<(), String>> {
    if args.first().is_none_or(|a| a != FINISH) { return None; }
    Some((|| {
        let [_, pid, target, archive, checksum, previous, tag] = args else { return Err("Argumentos de atualização inválidos".into()); };
        let pid = pid.to_string_lossy().parse::<u32>().map_err(|_| "PID de atualização inválido")?;
        let target = PathBuf::from(target);
        let archive = PathBuf::from(archive).canonicalize().map_err(|e| e.to_string())?;
        let tag = tag.to_str().ok_or("Tag inválida")?;
        if asset_name(tag).is_none_or(|n| archive.file_name().is_none_or(|f| f != n)) {
            return Err("Pacote não corresponde à Release Xiri".into());
        }
        let base = dirs::data_local_dir().ok_or("AppData Local indisponível")?
            .join("Xiri Launcher").join("xiri-self-updates");
        let folder = base.join(tag).canonicalize().map_err(|e| e.to_string())?;
        if !folder.starts_with(base.canonicalize().map_err(|e| e.to_string())?)
            || archive.parent() != Some(folder.as_path())
        { return Err("Pacote de atualização fora do cache Xiri".into()); }
        let helper = std::env::current_exe().map_err(|e| e.to_string())?.canonicalize().map_err(|e| e.to_string())?;
        if helper != folder.join("xiri-launcher-updater.exe") { return Err("Auxiliar de atualização em local inválido".into()); }
        let target = target.canonicalize().map_err(|e| e.to_string())?;
        if target.file_name().is_none_or(|n| !n.to_string_lossy().eq_ignore_ascii_case("xiri-launcher.exe")) {
            return Err("Este executável não é o Xiri Launcher".into());
        }
        let expected = checksum.to_str().ok_or("Checksum inválido")?;
        let previous = previous.to_str().ok_or("Identidade inválida")?;
        if expected.len() != 64 || previous.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit())
            || !previous.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("Digest inválido".into()); }
        if sha_file(&helper)? != previous || sha_file(&target)? != previous {
            return Err("Instalação do Xiri foi modificada durante a atualização".into());
        }
        if sha_file(&archive)? != expected { return Err("ZIP não corresponde ao checksum validado".into()); }
        archive_version(&archive, tag)?;
        let folder = target.parent().ok_or("Pasta do Xiri inválida")?;
        let version = std::fs::read_to_string(folder.join("VERSION")).map_err(|e| e.to_string())?;
        if version.trim().trim_start_matches('\u{feff}') != env!("CARGO_PKG_VERSION") {
            return Err("Versão instalada mudou antes da atualização".into());
        }
        if !crate::process::wait_for_exit(pid, std::time::Duration::from_secs(60)) {
            return Err("Xiri não fechou; atualização cancelada sem modificar arquivos".into());
        }
        crate::portable::unpack_over(&archive, &folder.join("XiriUpdateStaging"), folder,
            std::ffi::OsStr::new("xiri-launcher.exe"), &|file| {
                if file.file_name().is_some_and(|n| n == "xiri-launcher.exe") {
                    let mut b = [0u8; 2];
                    std::fs::File::open(file).and_then(|mut f| f.read_exact(&mut b)).map_err(|e| e.to_string())?;
                    if b != *b"MZ" { return Err("O novo Xiri não é executável válido".into()); }
                }
                Ok(())
            })?;
        let restart = std::process::Command::new(&target);
        crate::process::spawn_detached(restart).map_err(|e| format!("Xiri atualizado, mas não foi possível reabri-lo: {e}"))?;
        Ok(())
    })())
}
#[cfg(not(windows))]
pub fn finish(_: &[OsString]) -> Option<Result<(), String>> { None }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_release_without_zip_and_checksum() {
        let json = r#"[{"tag_name":"v0.1.0-beta.2","assets":[{"name":"xiri-launcher-0.1.0-beta.2-windows-x64-portable.zip","size":100}]}]"#;
        let release = release::parse_github_release_list(json, REPO, true).unwrap().unwrap();
        assert!(!suitable(&release));
        assert!(checksum_for("bad contents", "a.zip").is_err());
        assert!(asset_name("v../../a").is_none());
    }
    #[test]
    fn rejects_unsupported_updater_arguments() {
        assert!(finish(&[]).is_none());
        assert!(finish(&[FINISH.into()]).is_some_and(|r| r.is_err()) || !cfg!(windows));
    }
}
