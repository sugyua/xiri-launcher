//! Windows Destinia updater. The chosen directory is persisted in Xiri settings.
//! A destructive file overlay always requires a separate confirmation.
use crate::app::{Host, LauncherUi};
use crate::widgets::{self, Kind};
use xiri_launcher_core::protocol::Command;

impl LauncherUi {
    pub(crate) fn destinia_controls(&mut self, ui: &mut egui::Ui, host: &mut dyn Host) {
        if !self.caps.manage_apps {
            return;
        }
        let info = self.destinia_release.clone();
        let folder = self.settings.destinia_folder.clone();
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Destinia · atualizações");
                match &info {
                    Some(Ok(release)) => { ui.label(format!("Versão disponível: {}", release.tag)); }
                    Some(Err(error)) => { ui.label(error); }
                    None => { ui.label("Consultando Releases públicas do GitHub"); }
                }
                if self.destinia_busy {
                    // No animated spinner: the egui UI test harness must reach a stable frame.
                    ui.label("Operação em andamento…");
                } else if widgets::button(ui, "Verificar", Kind::Secondary, 0.0).clicked() {
                    self.destinia_busy = true;
                    self.send(Command::DestiniaCheck);
                }
            });
            ui.horizontal_wrapped(|ui| {
                match &folder {
                    Some(dir) => { ui.small(format!("Pasta do jogo: {dir}")); }
                    None => { ui.small("Selecione a pasta que contém VERSION, game, host e host_zoom"); }
                }
                if widgets::button(ui, "Escolher pasta", Kind::Secondary, 0.0).clicked()
                    && let Some(dir) = host.pick_folder(self.settings.destinia_folder.as_deref())
                {
                    self.settings.destinia_folder = Some(dir);
                }
                if let Some(Ok(_)) = &info
                    && folder.is_some() && !self.destinia_busy
                    && widgets::button(ui, "Baixar e instalar atualização", Kind::Primary, 0.0).clicked()
                {
                    self.destinia_confirm = true;
                }
            });
            ui.small("O Xiri baixa a Release Windows, valida SHA-256, substitui os arquivos do pacote e mantém arquivos extras e saves. Backup antes de substituir. Feche o jogo.");
        });
        ui.add_space(9.0);

        if self.destinia_confirm {
            let mut approve = false;
            let mut cancel = false;
            egui::Window::new("Confirmar atualização do Destinia")
                .id(egui::Id::new("xiri-destinia-confirm-update"))
                .collapsible(false)
                .resizable(false)
                .default_width(550.0)
                .show(ui.ctx(), |dialog| {
                    if let Some(Ok(release)) = &info {
                        dialog.label(format!("Versão a instalar: {}", release.tag));
                    }
                    if let Some(dir) = &folder {
                        dialog.label(format!("Destino: {dir}"));
                    }
                    dialog.label("A instalação substituirá arquivos com o mesmo caminho presentes no ZIP e adicionará novos arquivos.");
                    dialog.label("Arquivos não incluídos na Release, incluindo game, saves e dados pessoais, serão preservados. Será criado um backup dos arquivos substituídos para reversão.");
                    dialog.label("Feche todas as janelas do Destinia antes de continuar.");
                    dialog.horizontal(|ui| {
                        if widgets::button(ui, "Cancelar", Kind::Secondary, 0.0).clicked() { cancel = true; }
                        if widgets::button(ui, "Confirmar instalação", Kind::Primary, 0.0).clicked() { approve = true; }
                    });
                });
            if cancel {
                self.destinia_confirm = false;
            }
            if approve {
                self.destinia_confirm = false;
                if let (Some(Ok(release)), Some(dir)) = (info, folder) {
                    self.destinia_busy = true;
                    self.send(Command::DestiniaInstall { folder: dir, tag: release.tag });
                }
            }
        }
    }
}
