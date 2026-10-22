//! Owned GitHub catalog and the versioned Xiri package contract.
use serde::{Deserialize, Serialize};

pub const OWNER: &str = "sugyua";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub repo: String,
    pub color: [u8; 3],
    pub ink_light: [u8; 3],
    pub ink_dark: [u8; 3],
    #[serde(default)]
    pub default_branch: String,
    #[serde(default)]
    pub manifest: Option<XiriManifest>,
    #[serde(default)]
    pub manifest_error: Option<String>,
}

impl AppInfo {
    pub fn trusted(&self) -> bool {
        valid_entry_path(&self.id) && !self.id.contains('/') && self.repo.eq_ignore_ascii_case(&format!("{OWNER}/{}", self.id))
    }
    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }
    pub fn releases_url(&self) -> String {
        format!("{}/releases", self.repo_url())
    }
    pub fn website(&self) -> String {
        self.repo_url()
    }
    pub fn stems(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.id.as_str())
    }
}

impl From<&crate::CraftApp> for AppInfo {
    fn from(a: &crate::CraftApp) -> Self {
        Self {
            id: a.id.into(),
            name: a.name.into(),
            tagline: a.tagline.into(),
            repo: a.repo.into(),
            color: a.color,
            ink_light: a.ink_light,
            ink_dark: a.ink_dark,
            default_branch: "main".into(),
            manifest: None,
            manifest_error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Exe,
    Powershell,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub kind: EntryKind,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsPackage {
    pub asset: String,
    pub entry: PackageEntry,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    #[default]
    Program,
    Game,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct XiriManifest {
    pub schema_version: u32,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub category: Category,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub windows: Option<WindowsPackage>,
}
fn yes() -> bool {
    true
}

pub fn valid_entry_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 240 || path.contains(['\\', ':', '\0']) {
        return false;
    }
    path.split('/').all(|part| {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        !part.is_empty()
            && part != "."
            && part != ".."
            && !part.ends_with(['.', ' '])
            && !part.chars().any(|c| c.is_control() || "<>\"|?*".contains(c))
            && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes().get(3).is_some_and(|b| b.is_ascii_digit()))
    })
}

pub fn parse_manifest(bytes: &[u8]) -> Result<XiriManifest, String> {
    if bytes.len() > 65536 {
        return Err("O manifesto excede 64 KiB".into());
    }
    let m: XiriManifest = serde_json::from_slice(bytes).map_err(|e| format!("Manifesto inválido: {e}"))?;
    if m.schema_version != 1 {
        return Err("Versão de manifesto não suportada".into());
    }
    if m.icon.as_ref().is_some_and(|icon| !valid_entry_path(icon) || !icon.ends_with(".png")) {
        return Err("Ícone deve ser um PNG com caminho relativo no repositório".into());
    }
    if let Some(w) = &m.windows {
        if !valid_entry_path(&w.entry.path) || !valid_entry_path(&w.asset) || w.asset.contains('/') || !w.asset.ends_with(".zip") {
            return Err("Caminho ou pacote Windows inválido".into());
        }
        let extension = match w.entry.kind {
            EntryKind::Exe => ".exe",
            EntryKind::Powershell => ".ps1",
        };
        if !w.entry.path.to_ascii_lowercase().ends_with(extension) {
            return Err("Tipo de entrada incompatível com a extensão".into());
        }
    }
    Ok(m)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub owner: Owner,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default_branch: String,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub private: bool,
}

pub fn catalog(repositories: Vec<Repository>) -> Vec<AppInfo> {
    let mut apps: Vec<_> = repositories
        .into_iter()
        .filter(|r| {
            r.owner.login.eq_ignore_ascii_case(OWNER)
                && r.full_name == format!("{}/{}", r.owner.login, r.name)
                && !r.fork
                && !r.archived
                && !r.private
                && !["xiri-launcher", OWNER].iter().any(|n| r.name.eq_ignore_ascii_case(n))
                && valid_entry_path(&r.name)
                && !r.name.contains('/')
        })
        .map(|r| AppInfo {
            id: r.name.clone(),
            name: r.name,
            tagline: r.description.unwrap_or_default(),
            repo: r.full_name,
            color: [113, 83, 230],
            ink_light: [91, 57, 190],
            ink_dark: [181, 157, 255],
            default_branch: r.default_branch,
            manifest: None,
            manifest_error: None,
        })
        .collect();
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps.dedup_by(|a, b| a.repo.eq_ignore_ascii_case(&b.repo));
    apps
}

pub fn package_asset<'a>(app: &AppInfo, release: &'a crate::Release) -> Option<&'a crate::Asset> {
    let windows = app.manifest.as_ref()?.windows.as_ref()?;
    let checksum = release.assets.iter().any(|a| a.name == "SHA256SUMS.txt");
    if !checksum || app.manifest_error.is_some() {
        return None;
    }
    release.assets.iter().find(|a| a.name == windows.asset)
}

pub fn trusted_download_url(url: &str, repo: &str) -> bool {
    if url.len() > 16_384 || url.chars().any(|c| c.is_whitespace() || c.is_control()) || url.contains('\\') {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    match host {
        "github.com" => crate::release::download_url_under(url, &crate::release::download_prefix(repo)),
        "release-assets.githubusercontent.com" | "objects.githubusercontent.com" | "github-releases.githubusercontent.com" => true,
        _ => false,
    }
}
