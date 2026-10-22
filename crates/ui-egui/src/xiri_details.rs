//! Repository details, using the same centered sheet and design tokens as the upstream UI.
use crate::app::LauncherUi;
use crate::theme::{self, Tokens};
use crate::widgets::{self, Kind};

impl LauncherUi {
    pub(crate) fn xiri_details_drawer(&mut self, ctx: &egui::Context) {
        let Some(id) = self.detail.clone() else { return };
        let Some(entry) = self.entry(&id).cloned() else {
            self.detail = None;
            return;
        };
        let t = Tokens::get(ctx);
        let size = crate::sheet::size(ctx, egui::vec2(1100.0, 760.0));
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("xiri-details").with(&id)).frame(crate::sheet::frame(&t));
        let response = modal.show(ctx, |ui| {
            ui.set_width(size.x);
            ui.set_max_height(size.y);
            crate::sheet::hero(ui, Some(crate::icons::artcraft_mark()), &entry.app.name, &entry.app.tagline, |ui| {
                if widgets::button(ui, tl!("Close"), Kind::Secondary, 80.0).clicked() {
                    close = true;
                }
            });
            egui::Frame::new().inner_margin(theme::PAD as i8).show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(size.y - 150.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        widgets::hud_link(ui, tl!("GitHub"), &entry.app.repo_url());
                        widgets::hud_link(ui, tl!("Downloads"), &entry.app.releases_url());
                    });
                    ui.add_space(18.0);
                    ui.label(egui::RichText::new(&entry.app.tagline).size(16.0).color(t.ink));
                    if let Some(error) = &entry.app.manifest_error {
                        ui.colored_label(t.accent, error);
                    }
                    if let Some(m) = &entry.app.manifest {
                        let category = match m.category {
                            xiri_launcher_core::xiri::Category::Game => tl!("Game"),
                            _ => tl!("Program"),
                        };
                        widgets::hud(ui, category, t.muted);
                    }
                    ui.add_space(18.0);
                    let status = entry.status(self.platform, self.caps.manage_apps);
                    self.actions_in(ui, &entry, &status, 300.0, true);
                    ui.add_space(20.0);
                    if entry.app.manifest.as_ref().and_then(|m| m.windows.as_ref()).is_none() {
                        ui.label(tl!("Automatic installation is not configured. See the repository for requirements or associate a local program."));
                    }
                    if let Some(release) = entry.release() {
                        ui.separator();
                        widgets::hud(ui, &format!("{} {}", tl!("Version"), release.version_label()), t.muted);
                        if let Some(at) = release.published_at {
                            ui.label(crate::views::date_text(at));
                        }
                        ui.add_space(12.0);
                        ui.label(&release.notes);
                    } else {
                        ui.label(tl!("No release published yet"));
                    }
                });
            });
        });
        if close || response.should_close() {
            self.detail = None;
        }
    }
}
