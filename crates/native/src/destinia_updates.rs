//! Read/download the owner's private Destinia releases using an existing gh login.
//! Never ask for, persist or log a token. Never replace installed files or extract APK assets.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use xiri_launcher_core::protocol::DestiniaReleaseInfo;

const REPO: &str = "sugyua/destinia";
const AUTH_HELP: &str =
    "Destinia e privado. Instale o GitHub CLI (gh) e execute 'gh auth login' uma vez no seu Windows. O Xiri nao armazena senhas ou tokens.";

#[derive(Deserialize)]
struct Asset {
    name: String,
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

/// Choose only a release with the exact matching Windows ZIP and checksum asset.
pub fn parse_latest(data: &str) -> Result<DestiniaReleaseInfo, String> {
    let releases: Vec<Entry> = serde_json::from_str(data).map_err(|_| "Resposta de Releases invalida".to_string())?;
    for release in releases {
        let Some(asset) = asset_name(&release.tag_name) else { continue };
        if release.draft {
            continue;
        }
        if release.assets.iter().any(|a| a.name == asset) && release.assets.iter().any(|a| a.name == format!("{asset}.sha256")) {
            return Ok(DestiniaReleaseInfo { tag: release.tag_name, asset });
        }
    }
    Err("Nenhuma Release Windows com ZIP e SHA-256 correspondente foi encontrada.".into())
}

fn gh(arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("gh")
        .args(arguments)
        .env("GH_PROMPT_DISABLED", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|_| AUTH_HELP.to_string())?;
    if !output.status.success() {
        // Never display stderr: gh can include private repository URLs or auth details.
        return Err(AUTH_HELP.into());
    }
    Ok(output.stdout)
}

pub fn check() -> Result<DestiniaReleaseInfo, String> {
    let bytes = gh(&["api", "repos/sugyua/destinia/releases?per_page=30"])?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "Resposta GitHub nao e UTF-8")?;
    parse_latest(text)
}

fn read_digest(file: &Path) -> Result<String, String> {
    let sum = fs::read_to_string(file).map_err(|_| "Checksum da Release ausente")?;
    let hash = sum.split_whitespace().next().ok_or("Checksum da Release vazio")?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SHA-256 publicado invalido".into());
    }
    Ok(hash.to_ascii_lowercase())
}

fn actual_digest(file: &Path) -> Result<String, String> {
    let mut f = File::open(file).map_err(|e| format!("ZIP nao pode ser lido: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = f.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_zip(file: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut zip = zip::ZipArchive::new(File::open(file).map_err(|e| e.to_string())?).map_err(|_| "ZIP Windows invalido")?;
        if zip.len() > 2000 {
            return Err("ZIP com arquivos demais".into());
        }
        let mut normal = false;
        let mut zoom = false;
        for i in 0..zip.len() {
            let entry = zip.by_index(i).map_err(|_| "Entrada ZIP invalida")?;
            let name = entry.name();
            if !xiri_launcher_core::xiri::valid_entry_path(name.trim_end_matches('/')) {
                return Err("ZIP com caminho inseguro".into());
            }
            if name.ends_with("/Destinia.exe") {
                normal = true;
            }
            if name.ends_with("/Destinia Zoom.exe") {
                zoom = true;
            }
        }
        if !normal || !zoom {
            return Err("ZIP Windows sem os dois executaveis da raiz".into());
        }
    }
    #[cfg(not(windows))]
    let _ = file;
    Ok(())
}

/// Only downloads to Xiri's dedicated data folder. No file in the user's game is touched.
pub fn download(root: &Path, release: &DestiniaReleaseInfo) -> Result<PathBuf, String> {
    if asset_name(&release.tag) != Some(release.asset.clone()) {
        return Err("Release desconhecida".into());
    }
    if root.as_os_str().is_empty() {
        return Err("Diretorio de dados indisponivel".into());
    }
    let downloads = root.join("destinia-downloads");
    fs::create_dir_all(&downloads).map_err(|e| e.to_string())?;
    let destination = downloads.join(&release.tag);
    let target = destination.join(&release.asset);
    if target.exists() {
        let digest = destination.join(format!("{}.sha256", release.asset));
        if read_digest(&digest)? == actual_digest(&target)? {
            verify_zip(&target)?;
            return Ok(target);
        }
        return Err("Download anterior difere do SHA-256. Remova a pasta antiga manualmente antes de baixar novamente.".into());
    }
    if destination.exists() {
        return Err("Ja existe uma pasta da mesma versao. Inspecione-a antes de continuar.".into());
    }
    let staging = tempfile::tempdir_in(&downloads).map_err(|e| e.to_string())?;
    let folder = staging.path().to_str().ok_or("Caminho Windows invalido")?;
    for pattern in [&release.asset, &format!("{}.sha256", release.asset)] {
        gh(&["release", "download", &release.tag, "--repo", REPO, "--pattern", pattern, "--dir", folder])?;
    }
    let temporary = staging.path().join(&release.asset);
    if read_digest(&staging.path().join(format!("{}.sha256", release.asset)))? != actual_digest(&temporary)? {
        return Err("SHA-256 do download nao confere: nada foi instalado.".into());
    }
    verify_zip(&temporary)?;
    fs::rename(staging.path(), &destination).map_err(|e| format!("Nao foi possivel finalizar download: {e}"))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_properly_versioned_release_with_both_assets() {
        let good = r#"[{"tag_name":"v0.9.2-beta","assets":[{"name":"destinia-0.9.2-beta-windows-x64.zip"},{"name":"destinia-0.9.2-beta-windows-x64.zip.sha256"}]}]"#;
        let r = parse_latest(good).unwrap();
        assert_eq!(r.tag, "v0.9.2-beta");
        assert_eq!(r.asset, "destinia-0.9.2-beta-windows-x64.zip");
        assert!(parse_latest(r#"[{"tag_name":"v0.9.2-beta","assets":[{"name":"destinia-0.9.2-beta-windows-x64.zip"}]}]"#).is_err());
        assert!(parse_latest(r#"[{"tag_name":"v../../escape","assets":[]}]"#).is_err());
        assert!(parse_latest(r#"[{"tag_name":"v0.9.2-beta","draft":true,"assets":[{"name":"destinia-0.9.2-beta-windows-x64.zip"},{"name":"destinia-0.9.2-beta-windows-x64.zip.sha256"}]}]"#).is_err());
    }
    #[test]
    fn does_not_touch_installation_on_invalid_release() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("save.dat");
        fs::write(&marker, b"keep").unwrap();
        let fake = DestiniaReleaseInfo { tag: "../../escape".into(), asset: "bogus.zip".into() };
        assert!(download(dir.path(), &fake).is_err());
        assert_eq!(fs::read(marker).unwrap(), b"keep");
    }
}
