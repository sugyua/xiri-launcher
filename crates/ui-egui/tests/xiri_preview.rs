//! A deterministic Xiri catalog: verify installation routing and optionally render the real UI.
use egui_kittest::{Harness, kittest::Queryable};
use std::cell::RefCell;
use std::rc::Rc;
use xiri_launcher_core::protocol::{Backend, Capabilities, Command, Event, LatestRelease, Origin};
use xiri_launcher_core::xiri::{AppInfo, parse_manifest};
use xiri_launcher_core::{Arch, Asset, Os, Platform, Release, version};
use xiri_launcher_ui_egui::settings::ViewMode;
use xiri_launcher_ui_egui::{Host, LauncherUi, Settings, TitleBar};

struct Window;
impl Host for Window {
    fn title_bar(&self) -> TitleBar {
        TitleBar::Custom
    }
}
struct Catalog {
    apps: Vec<AppInfo>,
    events: Vec<Event>,
    sent: Rc<RefCell<Vec<Command>>>,
}
impl Backend for Catalog {
    fn dynamic_catalog(&self) -> bool {
        true
    }
    fn catalog(&self) -> Vec<AppInfo> {
        self.apps.clone()
    }
    fn send(&mut self, command: Command) {
        self.sent.borrow_mut().push(command);
    }
    fn poll(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities { manage_apps: true, reveal_label: "Show in file manager", self_update: false }
    }
    fn platform(&self) -> Option<Platform> {
        Some(Platform { os: Os::Windows, arch: Arch::X64 })
    }
    fn now(&self) -> i64 {
        1_792_703_000
    }
}
fn app(id: &str, name: &str, description: &str) -> AppInfo {
    let mut app = AppInfo::from(&xiri_launcher_core::SUITE[0]);
    app.id = id.into();
    app.name = name.into();
    app.tagline = description.into();
    app.repo = format!("sugyua/{id}");
    app.color = [113, 83, 230];
    app.ink_light = [91, 57, 190];
    app.ink_dark = [181, 157, 255];
    app
}

#[test]
fn owned_catalog_renders_and_install_bypasses_legacy_folder_picker() {
    xiri_launcher_ui_egui::i18n::set_test_lang(xiri_launcher_ui_egui::i18n::Lang::EN);
    for (view, name) in [(ViewMode::Grid, "xiri-grade"), (ViewMode::Table, "xiri-tabela")] {
        let mut sensor = app("sensor-panel", "Sensor Panel", "CPU, GPU, memória e FPS no monitor secundário 1280 × 400.");
        sensor.manifest=Some(parse_manifest(br#"{"schema_version":1,"windows":{"asset":"sensor-panel-0.1.0-beta-windows-x64.zip","entry":{"kind":"powershell","path":"MinhaVersao/Painel.ps1"}}}"#).unwrap());
        let apps = vec![
            app("destinia", "Destinia", "RPG para Windows. Consulte os requisitos no repositório."),
            app("Destinia_PCport", "Destinia_PCport", "Port para PC do RPG Destinia."),
            sensor,
        ];
        let release = Release {
            tag: "v0.1.0-beta".into(),
            version: version::parse("0.1.0-beta"),
            assets: vec![
                Asset { name: "sensor-panel-0.1.0-beta-windows-x64.zip".into(), size: 44059, ..Asset::default() },
                Asset { name: "SHA256SUMS.txt".into(), size: 107, ..Asset::default() },
            ],
            ..Release::default()
        };
        let mut events = vec![Event::Latest { app: "sensor-panel".into(), latest: LatestRelease::Found(release), from: Origin::GitHub }];
        for app in &apps {
            events.push(Event::Detected { app: app.id.clone(), installed: None });
        }
        for id in ["destinia", "Destinia_PCport"] {
            events.push(Event::Latest { app: id.into(), latest: LatestRelease::None, from: Origin::GitHub });
        }
        let sent = Rc::new(RefCell::new(Vec::new()));
        let mut backend = Some(Catalog { apps, events, sent: sent.clone() });
        let settings =
            Settings { view, language: "pt-BR".into(), notify: false, theme: xiri_launcher_ui_egui::theme::ThemeChoice::Dark, ..Settings::default() };
        let mut harness = Harness::builder().with_size(egui::vec2(1180.0, 850.0)).with_theme(egui::Theme::Dark).build_ui_state(
            move |ui, state: &mut Option<LauncherUi>| {
                if let Some(backend) = backend.take() {
                    *state = Some(LauncherUi::new(ui.ctx(), Box::new(backend), settings.clone()));
                }
                if let Some(launcher) = state.as_mut() {
                    launcher.logic(&ui.ctx().clone(), &mut Window);
                    launcher.ui(ui, &mut Window);
                }
            },
            None,
        );
        harness.run_steps(8);
        harness.get_by_label("Sensor Panel");
        harness.get_by_label("Instalar");
        if let Some(directory) = std::env::var_os("XIRI_LAUNCHER_SNAPSHOTS") {
            std::fs::create_dir_all(&directory).unwrap();
            harness.render().unwrap().save(std::path::Path::new(&directory).join(format!("{name}.png"))).unwrap();
        }
        harness.get_by_label("Instalar").click();
        harness.run_steps(3);
        assert!(sent.borrow().iter().any(|c| matches!(c,Command::Install {app} if app=="sensor-panel")));
    }
}
