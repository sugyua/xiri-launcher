//! Transactional public GitHub catalog, separate from installed applications.
use crate::http::{Fetched, Http};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use xiri_launcher_core::xiri::{AppInfo, Repository, catalog, parse_manifest};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CatalogCache {
    pub apps: Vec<AppInfo>,
    pub pages: BTreeMap<usize, CachedPage>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedPage {
    pub text: String,
    pub etag: Option<String>,
}

pub fn fetch_pages(cache: &CatalogCache, mut get: impl FnMut(&str, Option<&str>) -> Result<Fetched, String>) -> Result<CatalogCache, String> {
    let mut next = CatalogCache::default();
    let mut repositories = Vec::new();
    for page in 1..=20 {
        let saved = cache.pages.get(&page);
        let fetched = get(&format!("https://api.github.com/users/sugyua/repos?per_page=100&page={page}"), saved.and_then(|p| p.etag.as_deref()))?;
        let value = match fetched {
            Fetched::Body { text, etag } => CachedPage { text, etag },
            Fetched::NotModified => saved.cloned().ok_or("GitHub retornou 304 sem cache local")?,
            Fetched::RateLimited { .. } => return Err("Limite do GitHub atingido. O catálogo salvo foi mantido.".into()),
            Fetched::NotFound => return Err("Perfil GitHub não encontrado".into()),
        };
        if value.text.len() > 4 * 1024 * 1024 {
            return Err("Resposta GitHub muito grande".into());
        }
        let rows: Vec<Repository> = serde_json::from_str(&value.text).map_err(|e| format!("Catálogo inválido: {e}"))?;
        let done = rows.len() < 100;
        repositories.extend(rows);
        next.pages.insert(page, value);
        if done {
            next.apps = catalog(repositories);
            return Ok(next);
        }
    }
    Err("O perfil excede o limite de 2.000 repositórios".into())
}

pub fn refresh(http: &Http, cache: &CatalogCache) -> Result<CatalogCache, String> {
    let mut next = fetch_pages(cache, |url, etag| http.get_api(url, etag))?;
    for app in &mut next.apps {
        // GitHub's HEAD alias avoids interpolating an untrusted branch into a URL.
        let url = format!("https://raw.githubusercontent.com/{}/HEAD/xiri.json", app.repo);
        match http.get_bytes(&url, 65536) {
            Ok(Some(bytes)) => match parse_manifest(&bytes) {
                Ok(m) => {
                    if let Some(title) = &m.title {
                        app.name = title.clone();
                    }
                    if let Some(description) = &m.description {
                        app.tagline = description.clone();
                    }
                    app.manifest = Some(m);
                }
                Err(e) => app.manifest_error = Some(e),
            },
            Ok(None) => {}
            Err(e) => app.manifest_error = Some(e),
        }
    }
    next.apps.retain(|a| a.manifest.as_ref().is_none_or(|m| m.visible));
    Ok(next)
}
