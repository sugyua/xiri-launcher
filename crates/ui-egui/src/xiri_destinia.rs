//! Compact Windows-only Destinia release panel. Never installs into game directories.
use crate::app::LauncherUi;
use crate::widgets::{self, Kind};
use xiri_launcher_core::protocol::Command;

impl LauncherUi {
    pub(crate) fn destinia_controls(&mut self, ui: &mut egui::Ui) {
        if !self.caps.manage_apps { return; }
        let info = self.destinia_release.clone();
        let downloaded = self.destinia_zip.is_some();
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Destinia");
                match info {
                    Some(Ok(release)) => {
                        ui.label(format!("Release disponível: {}", release.tag));
                        ui.label("(repositório privado)");
                        if !self.destinia_busy
                            && widgets::button(ui, "Baixar ZIP verificado", Kind::Primary, 0.0).clicked()
                        {
                            self.destinia_busy = true;
                            self.destinia_zip = None;
                            self.send(Command::DestiniaDownload);
                        }
                    }
                    Some(Err(error)) => {
                        ui.label(error);
                    }
                    None => {
                        ui.label("Consulte as versões sem abrir o GitHub");
                    }
                }
                if self.destinia_busy {
                    ui.spinner();
                    ui.label("Consultando ou baixando…");
                } else if widgets::button(ui, "Verificar versão", Kind::Secondary, 0.0).clicked() {
                    self.destinia_busy = true;
                    self.send(Command::DestiniaCheck);
                }
                if downloaded
                    && widgets::button(ui, "Abrir pasta do ZIP", Kind::Secondary, 0.0).clicked()
                {
                    self.send(Command::DestiniaReveal);
                }
            });
            ui.small("Download autenticado pelo GitHub CLI (gh), com SHA-256. A instalação não é automática; seus saves e arquivos game não são modificados.");
        });
    }
}
