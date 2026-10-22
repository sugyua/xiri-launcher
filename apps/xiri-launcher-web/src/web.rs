//! The browser backend and entry point.

use std::cell::RefCell;
use std::rc::Rc;

use xiri_launcher_core::protocol::{Backend, Capabilities, Command, Event, LatestRelease, Origin, Source};
use xiri_launcher_core::release::{self, Release};
use xiri_launcher_core::{Arch, Os, Platform};
use xiri_launcher_core::xiri::{AppInfo,Repository,parse_manifest,catalog};
use xiri_launcher_ui_egui::{Host, LauncherUi, Settings, i18n};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

const SETTINGS_KEY: &str = "settings";

/// Tell the UI the browser's languages (`?lang=ru` in the address first, as `XIRI_LAUNCHER_LOCALE`
/// does on the desktop) before the first frame, so the Automatic interface language follows the
/// browser; and show the loading text in that language.
fn detect_language() {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    let mut tags = Vec::new();

    // ?lang=xx in the URL takes priority.
    if let Ok(href) = window.location().href()
        && let Some(idx) = href.find('?')
    {
        for pair in href[idx + 1..].split('&') {
            if let Some((k, v)) = pair.split_once('=')
                && k == "lang"
                && !v.is_empty()
            {
                tags.push(v.to_string());
            }
        }
    }

    // Then the browser's preferred languages.
    let navigator = window.navigator();
    let langs: js_sys::Array = navigator.languages();
    for i in 0..langs.length() {
        if let Some(v) = langs.get(i).as_string() {
            tags.push(v);
        }
    }
    if let Some(lang) = navigator.language() {
        tags.push(lang);
    }
    i18n::set_system_locales(&tags);

    // Screen readers and spell-check follow the page language: match <html lang>
    // to what was detected (`en` when the browser asks for nothing supported).
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("lang", i18n::system_lang().code());
    }

    // Update loading text in the detected language.
    if let Some(el) = document.get_element_by_id("launcher_loading") {
        el.set_text_content(Some(i18n::tr(i18n::system_lang(), "Loading Xiri Launcher…")));
    }
}
pub fn start() {
    eframe::WebLogger::init(log::LevelFilter::Warn).ok();
    // Detect language before starting the UI.
    detect_language();
    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
        let Some(canvas) = document.get_element_by_id("launcher_canvas").and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok()) else {
            log::error!("no canvas");
            return;
        };
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    let ctx = cc.egui_ctx.clone();
                    let settings: Settings = cc.storage.and_then(|s| eframe::get_value(s, SETTINGS_KEY)).unwrap_or_default();
                    let backend = WebBackend::new(ctx.clone());
                    Ok(Box::new(WebApp { ui: LauncherUi::new(&ctx, Box::new(backend), settings) }))
                }),
            )
            .await;
        if let Some(loading) = document.get_element_by_id("launcher_loading") {
            loading.remove();
        }
        if let Err(e) = result {
            log::error!("couldn't start: {e:?}");
        }
    });
}

/// The browser: nothing of the desktop's, but it knows whether the visitor asked for fewer animations.
struct WebHost;

impl Host for WebHost {
    fn reduce_motion(&self) -> bool {
        web_sys::window().and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten()).is_some_and(|query| query.matches())
    }
}

struct WebApp {
    ui: LauncherUi,
}

impl eframe::App for WebApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.logic(ctx, &mut WebHost);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui.ui(ui, &mut WebHost);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.ui.raw_input(ctx, raw_input);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, SETTINGS_KEY, &self.ui.settings);
    }
}

struct WebBackend {
    events: Rc<RefCell<Vec<Event>>>, ctx: egui::Context, platform: Option<Platform>, checking: Rc<RefCell<bool>>,
}
impl WebBackend {
    fn new(ctx: egui::Context) -> Self {Self {events:Rc::default(),ctx,platform:visitor_platform(),checking:Rc::default()}}
    fn check(&self) {
        if self.checking.replace(true) {return;}
        let events=self.events.clone();let ctx=self.ctx.clone();let checking=self.checking.clone();
        wasm_bindgen_futures::spawn_local(async move {
            events.borrow_mut().push(Event::CheckStarted);
            match public_catalog().await {
                Ok(apps)=>{
                    events.borrow_mut().push(Event::CatalogLoaded {apps:apps.clone(),warning:None});ctx.request_repaint();
                    for app in apps {
                        let latest=latest(&app.repo,false).await.unwrap_or_else(|_|LatestRelease::Failed("GitHub indisponível ou limite de consultas atingido".into()));
                        events.borrow_mut().push(Event::Latest {app:app.id,latest,from:Origin::GitHub});ctx.request_repaint();
                    }
                },
                Err(error)=>events.borrow_mut().push(Event::CatalogLoaded {apps:Vec::new(),warning:Some(error)}),
            }
            events.borrow_mut().push(Event::CheckFinished {at:(js_sys::Date::now()/1000.0) as i64,source:Source::GitHubApi,rate_limited_until:None});
            *checking.borrow_mut()=false;ctx.request_repaint();
        });
    }
}
async fn text(url:&str,max:usize)->Result<Option<String>,String> {
    let window=web_sys::window().ok_or("Navegador indisponível")?;
    let request=web_sys::Request::new_with_str(url).map_err(|_|"URL inválida")?;
    let response:web_sys::Response=JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_|"Sem conexão")?.dyn_into().map_err(|_|"Resposta inválida")?;
    if response.status()==404 {return Ok(None);}
    if response.status()!=200 {return Err(format!("GitHub retornou HTTP {}",response.status()));}
    if response.headers().get("content-length").ok().flatten().and_then(|s|s.parse::<usize>().ok()).is_some_and(|n|n>max) {return Err("Resposta grande demais".into());}
    let text=JsFuture::from(response.text().map_err(|_|"Resposta ilegível")?).await.map_err(|_|"Resposta ilegível")?.as_string().ok_or("Resposta inválida")?;
    if text.len()>max {return Err("Resposta grande demais".into());}
    Ok(Some(text))
}
async fn public_catalog()->Result<Vec<AppInfo>,String> {
    let mut rows=Vec::new();let mut complete=false;
    for page in 1..=20 {
        let body=text(&format!("https://api.github.com/users/sugyua/repos?per_page=100&page={page}"),4*1024*1024).await?.ok_or("Perfil não encontrado")?;
        let page:Vec<Repository>=serde_json::from_str(&body).map_err(|e|e.to_string())?;
        let done=page.len()<100;rows.extend(page);if done {complete=true;break;}
    }
    if !complete {return Err("O catálogo excede 2.000 repositórios".into());}
    let mut apps=catalog(rows);
    for app in &mut apps {
        match text(&format!("https://raw.githubusercontent.com/{}/HEAD/xiri.json",app.repo),65536).await {
            Ok(Some(body))=>match parse_manifest(body.as_bytes()) {
                Ok(manifest)=>{if let Some(title)=&manifest.title {app.name=title.clone();}if let Some(description)=&manifest.description {app.tagline=description.clone();}app.manifest=Some(manifest);},
                Err(error)=>app.manifest_error=Some(error),
            },
            Ok(None)=>{},Err(error)=>app.manifest_error=Some(error),
        }
    }
    apps.retain(|a|a.manifest.as_ref().is_none_or(|m|m.visible));Ok(apps)
}

async fn latest(repo: &str, prereleases: bool) -> Result<LatestRelease, Option<i64>> {
    let url = if prereleases {
        format!("https://api.github.com/repos/{repo}/releases?per_page=30")
    } else {
        format!("https://api.github.com/repos/{repo}/releases/latest")
    };
    let window = web_sys::window().ok_or(None)?;
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_mode(web_sys::RequestMode::Cors);
    let request = web_sys::Request::new_with_str_and_init(&url, &init).map_err(|_| None)?;
    let _ = request.headers().set("Accept", "application/vnd.github+json");
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_| None)?.dyn_into().map_err(|_| None)?;
    match resp.status() {
        200 => {}
        404 => return Ok(LatestRelease::None),
        403 | 429 => {
            let reset = resp.headers().get("x-ratelimit-reset").ok().flatten().and_then(|r| r.parse().ok());
            return Err(Some(reset.unwrap_or(0)));
        }
        _ => return Err(None),
    }
    let text = JsFuture::from(resp.text().map_err(|_| None)?).await.map_err(|_| None)?.as_string().ok_or(None)?;
    if text.len()>4*1024*1024 {return Err(None);}
    let found: Option<Release> = if prereleases {
        release::parse_github_release_list(&text, repo, true).map_err(|_| None)?
    } else {
        Some(release::parse_github_release(&text, repo).map_err(|_| None)?)
    };
    Ok(found.map_or(LatestRelease::None, LatestRelease::Found))
}

/// The visitor's OS and CPU, as far as the browser says.
fn visitor_platform() -> Option<Platform> {
    let nav = web_sys::window()?.navigator();
    let ua = nav.user_agent().ok()?.to_lowercase();
    let os = if ua.contains("windows") {
        Os::Windows
    } else if ua.contains("mac os") || ua.contains("macintosh") {
        Os::MacOs
    } else if ua.contains("freebsd") {
        Os::FreeBsd
    } else if ua.contains("linux") && !ua.contains("android") {
        Os::Linux
    } else {
        return None;
    };
    let arch = if ua.contains("aarch64") || ua.contains("arm64") || ua.contains("armv8") {
        Arch::Arm64
    } else if ua.contains("riscv") {
        Arch::Riscv64
    } else {
        Arch::X64
    };
    Some(Platform { os, arch })
}

impl Backend for WebBackend {
    fn dynamic_catalog(&self)->bool {true}
    fn catalog(&self)->Vec<AppInfo> {Vec::new()}
    fn send(&mut self,command:Command) {if matches!(command,Command::CheckForUpdates {..}) {self.check();}}
    fn poll(&mut self)->Vec<Event> {std::mem::take(&mut *self.events.borrow_mut())}
    fn capabilities(&self)->Capabilities {Capabilities {manage_apps:false,reveal_label:"",self_update:false}}
    fn platform(&self)->Option<Platform> {self.platform}
    fn now(&self)->i64 {(js_sys::Date::now()/1000.0) as i64}
}
