use xiri_launcher_native::xiri_repositories::{fetch_pages, CatalogCache};
use xiri_launcher_native::http::Fetched;

#[test]
fn pagination_is_complete_before_cache_is_replaced() {
    let mut calls = 0;
    let result = fetch_pages(&CatalogCache::default(), |_, _| {
        calls += 1;
        if calls == 1 {
            let rows: Vec<_> = (0..100).map(|i| serde_json::json!({"name":format!("app-{i}"),"full_name":format!("sugyua/app-{i}"),"owner":{"login":"sugyua"}})).collect();
            Ok(Fetched::Body { text: serde_json::to_string(&rows).unwrap(), etag: Some("page1".into()) })
        } else { Err("offline".into()) }
    });
    assert!(result.is_err());
    assert_eq!(calls, 2);
}

#[test]
fn not_modified_needs_a_cached_page() {
    assert!(fetch_pages(&CatalogCache::default(), |_, _| Ok(Fetched::NotModified)).is_err());
}

#[test]
fn valid_empty_profile_is_a_complete_catalog() {
    let cache = fetch_pages(&CatalogCache::default(), |_, _| Ok(Fetched::Body { text: "[]".into(), etag: None })).unwrap();
    assert!(cache.apps.is_empty());
    let revalidated = fetch_pages(&cache, |_, _| Ok(Fetched::NotModified)).unwrap();
    assert!(revalidated.apps.is_empty());
}
