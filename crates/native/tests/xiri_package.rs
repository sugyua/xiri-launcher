#![cfg(windows)]
use xiri_launcher_native::xiri_package::extract_verified;
use std::io::{Write, Cursor};
use zip::write::SimpleFileOptions;

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files { writer.start_file(*name, SimpleFileOptions::default()).unwrap(); writer.write_all(data).unwrap(); }
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
