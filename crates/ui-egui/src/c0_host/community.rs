//! The upstream community blocks, gated as a unit so hidden cards leave no blank space.
use crate::{
    PdfCraftApp,
    theme::{self, Tokens},
    widgets,
};

pub(crate) fn home(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    if !app.chrome.community_links {
        return;
    }
    let t = Tokens::get(ui.ctx());
    if !app.community_card_closed {
        egui::Frame::NONE
            .fill(t.card)
            .stroke(egui::Stroke::new(1.0, t.border))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    widgets::artcraft_mark(ui, 28.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(tl!("Join the ArtCraft community")).font(theme::semibold(15.0)));
                        ui.label(egui::RichText::new(tl!("Get help, share feedback and follow development on Discord.")).color(t.text_muted));
                    });
                    // Closed for this run only: the card is back on the next launch.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if crate::icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                            app.community_card_closed = true;
                        }
                    });
                });
                ui.add_space(8.0);
                if let Some(cmd) = widgets::community_links(ui) {
                    app.execute(cmd);
                }
            });
        ui.add_space(22.0);
    }
}

pub(crate) fn about(app: &PdfCraftApp, ui: &mut egui::Ui) -> Option<&'static str> {
    if !app.chrome.community_links {
        ui.label("By the PdfCraft contributors (see Contributors).");
        return None;
    }
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!("Part of")).color(t.text_muted));
        widgets::artcraft_logo(ui, 16.0);
    });
    ui.add_space(6.0);
    widgets::community_links(ui)
}
