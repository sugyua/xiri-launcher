#![cfg(windows)]
use std::io::{Cursor, Write};
use xiri_launcher_native::xiri_package::extract_verified;
use zip::write::SimpleFileOptions;

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files {
        writer.start_file(*name, SimpleFileOptions::default()).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn archive_rejects_traversal_and_case_collisions_before_writing() {
    for files in [vec![("../escape.exe", b"x".as_slice())], vec![("app.exe", b"x".as_slice()), ("APP.exe", b"x".as_slice())]] {
        let dir = tempfile::tempdir().unwrap();
        assert!(extract_verified(Cursor::new(archive(&files)), dir.path()).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

#[test]
fn valid_archive_is_extracted_without_running_anything() {
    let dir = tempfile::tempdir().unwrap();
    extract_verified(Cursor::new(archive(&[("bin/app.exe", b"not-an-executable")])), dir.path()).unwrap();
    assert_eq!(std::fs::read(dir.path().join("bin/app.exe")).unwrap(), b"not-an-executable");
}

#[test]
fn powershell_entry_uses_an_ordinary_windows_path() {
    let dir = tempfile::tempdir().unwrap();
    let body = br#"{"schema_version":1,"windows":{"asset":"app.zip","entry":{"kind":"powershell","path":"app.ps1"}}}"#;
    std::fs::write(dir.path().join("xiri.json"), body).unwrap();
    std::fs::write(dir.path().join("app.ps1"), b"# fixture, never executed").unwrap();
    let manifest = xiri_launcher_core::xiri::parse_manifest(body).unwrap();
    let target = xiri_launcher_native::xiri_package::validate_package(dir.path(), &manifest).unwrap();
    assert!(!target.to_string_lossy().starts_with("\\\\?\\"));
}
