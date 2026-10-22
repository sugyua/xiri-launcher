use xiri_launcher_core::xiri::{parse_manifest, valid_entry_path, Repository, catalog};

#[test]
fn downloads_stay_in_the_owners_release_or_githubs_asset_cdn() {
    use xiri_launcher_core::xiri::trusted_download_url;
    assert!(trusted_download_url("https://github.com/sugyua/sensor-panel/releases/download/v1/app.zip","sugyua/sensor-panel"));
    assert!(trusted_download_url("https://release-assets.githubusercontent.com/a?signature=abc","sugyua/sensor-panel"));
    for url in ["http://github.com/sugyua/sensor-panel/releases/download/v1/a.zip","https://github.com/other/app/releases/download/v1/a.zip","https://release-assets.githubusercontent.com.evil.test/a","https://release-assets.githubusercontent.com@evil.test/a","https://github.com/sugyua/sensor-panel/releases/download/../../../other/a.zip"] {
        assert!(!trusted_download_url(url,"sugyua/sensor-panel"),"{url}");
    }
}

#[test]
fn package_contract_validates_explicit_entry_and_schema() {
    let m = parse_manifest(br#"{"schema_version":1,"title":"Sensor Panel","category":"program","windows":{"asset":"sensor-panel-windows-x64.zip","entry":{"kind":"powershell","path":"MinhaVersao/Painel.ps1"}}}"#).unwrap();
    assert_eq!(m.title.as_deref(), Some("Sensor Panel"));
    assert!(m.windows.is_some());
    assert!(parse_manifest(br#"{"schema_version":2}"#).is_err());
    assert!(parse_manifest(br#"{"schema_version":1}"#).unwrap().windows.is_none());
    assert!(parse_manifest(br#"{"schema_version":1,"windows":{"asset":"x.zip","entry":{"kind":"exe","path":"../x.exe"}}}"#).is_err());
    assert!(parse_manifest(br#"{"schema_version":1,"icon":"../icon.png"}"#).is_err());
    assert!(parse_manifest(br#"{"schema_version":1,"icon":"icon.svg"}"#).is_err());
}

#[test]
fn windows_paths_cannot_escape_or_alias_another_file() {
    for path in ["", "../x", "/x", "C:\\x", "\\\\server\\x", "file:stream", "a/../../x", "CON", "aux.txt", "a./x", "x /y", "a//b", "a\\b.exe", "a/./b", "NUL.exe"] {
        assert!(!valid_entry_path(path), "{path}");
    }
    assert!(valid_entry_path("MinhaVersao/Painel.ps1"));
    assert!(valid_entry_path("Programa com espaço/app.exe"));
}

#[test]
fn catalog_filters_owned_repositories_without_name_guessing() {
    let rows: Vec<Repository> = serde_json::from_value(serde_json::json!([
        {"name":"destinia","full_name":"sugyua/destinia","owner":{"login":"sugyua"}},
        {"name":"Destinia_PCport","full_name":"sugyua/Destinia_PCport","owner":{"login":"sugyua"},"description":null},
        {"name":"fork","full_name":"sugyua/fork","owner":{"login":"sugyua"},"fork":true},
        {"name":"old","full_name":"sugyua/old","owner":{"login":"sugyua"},"archived":true},
        {"name":"xiri-launcher","full_name":"sugyua/xiri-launcher","owner":{"login":"sugyua"}},
        {"name":"sugyua","full_name":"sugyua/sugyua","owner":{"login":"sugyua"}},
        {"name":"bad","full_name":"other/bad","owner":{"login":"other"}}
    ])).unwrap();
    let apps = catalog(rows);
    assert_eq!(apps.len(), 2);
    assert!(apps.iter().any(|a| a.id == "destinia"));
    assert!(apps.iter().any(|a| a.id == "Destinia_PCport"));
}
