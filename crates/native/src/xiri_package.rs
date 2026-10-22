//! Portable packages: preflight every path and size before writing, never execute during install.
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use xiri_launcher_core::xiri::{valid_entry_path, parse_manifest, XiriManifest};

pub fn extract_verified<R: Read + Seek>(reader: R, destination: &Path) -> Result<(), String> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
    if zip.len() > 20_000 { return Err("Pacote com arquivos demais".into()); }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for index in 0..zip.len() {
        let file = zip.by_index(index).map_err(|e| e.to_string())?;
        let name = file.name().trim_end_matches('/');
        if !valid_entry_path(name) || !names.insert(name.to_lowercase()) || file.is_symlink() {
            return Err("ZIP contém caminho inválido, link ou nomes conflitantes".into());
        }
        total = total.checked_add(file.size()).ok_or("Tamanho inválido")?;
        if total > 4 * 1024 * 1024 * 1024 || file.size() > file.compressed_size().max(1).saturating_mul(200) {
            return Err("ZIP excede os limites de extração".into());
        }
        let mut prefix = String::new();
        for component in name.split('/').take(name.split('/').count().saturating_sub(1)) {
            if !prefix.is_empty() { prefix.push('/'); }
            prefix.push_str(component);
            if names.contains(&format!("file:{}", prefix.to_lowercase())) { return Err("Arquivo usado como diretório".into()); }
        }
        if !file.is_dir() {
            if names.iter().any(|n| n.starts_with(&format!("{}/", name.to_lowercase()))) { return Err("Conflito entre arquivo e diretório".into()); }
            names.insert(format!("file:{}", name.to_lowercase()));
        }
    }
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).map_err(|e| e.to_string())?;
        let target = destination.join(file.name());
        if file.is_dir() { std::fs::create_dir_all(&target).map_err(|e| e.to_string())?; continue; }
        if let Some(parent) = target.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(&target).map_err(|e| e.to_string())?;
        let actual = std::io::copy(&mut file, &mut output).map_err(|e| e.to_string())?;
        if actual != file.size() { return Err("Arquivo ZIP incompleto".into()); }
    }
    Ok(())
}

pub fn validate_package(root: &Path, expected: &XiriManifest) -> Result<PathBuf, String> {
    let file = std::fs::File::open(root.join("xiri.json")).map_err(|_| "O pacote precisa conter xiri.json na raiz")?;
    let mut bytes = Vec::new();
    file.take(65537).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let bundled = parse_manifest(&bytes)?;
    if bundled.windows != expected.windows { return Err("O contrato do pacote difere do manifesto remoto; atualize o catálogo".into()); }
    let entry = bundled.windows.as_ref().ok_or("Sem ponto de entrada Windows")?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let target = root.join(&entry.entry.path).canonicalize().map_err(|_| "O ponto de entrada não existe no pacote")?;
    if !target.starts_with(&root) || !target.is_file() { return Err("Ponto de entrada fora do pacote".into()); }
    Ok(target)
}
