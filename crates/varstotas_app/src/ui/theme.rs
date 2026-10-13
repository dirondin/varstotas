//! Light, friendly classic look.

use egui::{Color32, Context, CornerRadius, Frame, Margin, Stroke, Theme, Visuals};

pub const BAR_BG: Color32 = Color32::from_rgb(0xF3, 0xF4, 0xF6);
pub const TOOLBAR_BG: Color32 = Color32::from_rgb(0xEC, 0xEE, 0xF1);
pub const TRAY_BG: Color32 = Color32::from_rgb(0xF7, 0xF8, 0xFA);
pub const BORDER: Color32 = Color32::from_rgb(0xD3, 0xD7, 0xDD);
pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x7D, 0xE1);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xD6, 0xE6, 0xFA);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x6B, 0x72, 0x7D);

/// System fonts with arrows / ⌘ ⇧ etc., appended as fallbacks (egui's bundled fonts lack them).
const SYMBOL_FONTS: &[&str] = &[
    "/System/Library/Fonts/Apple Symbols.ttf",
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "C:\\Windows\\Fonts\\seguisym.ttf",
    "C:\\Windows\\Fonts\\segoeui.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

fn install_symbol_fallback(ctx: &Context) {
    let Some((path, bytes)) = SYMBOL_FONTS
        .iter()
        .find_map(|p| std::fs::read(p).ok().map(|b| (*p, b)))
    else {
        log::warn!("no symbol fallback font found; arrows may render as boxes");
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("symbols".into(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
    for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(fam).or_default().push("symbols".into());
    }
    ctx.set_fonts(fonts);
    log::debug!("symbol fallback font: {path}");
}

pub fn apply(ctx: &Context) {
    install_symbol_fallback(ctx);
    ctx.set_theme(egui::ThemePreference::Light);
    let mut v = Visuals::light();
    v.panel_fill = BAR_BG;
    v.window_fill = Color32::WHITE;
    v.extreme_bg_color = Color32::WHITE;
    v.faint_bg_color = Color32::from_rgb(0xF0, 0xF2, 0xF5);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xE9, 0xEC, 0xF0);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0xE4, 0xE7, 0xEC);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xDD, 0xE3, 0xEB);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0xB8, 0xC2, 0xD0));
    v.widgets.active.weak_bg_fill = ACCENT_SOFT;
    ctx.set_visuals_of(Theme::Light, v);
    ctx.style_mut_of(Theme::Light, |s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
        s.spacing.menu_margin = Margin::same(6);
        s.spacing.interact_size.y = 22.0;
    });
}

pub fn bar_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(BAR_BG)
        .inner_margin(Margin::symmetric(8, 3))
        .stroke(Stroke::new(1.0, BORDER))
}

pub fn toolbar_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(TOOLBAR_BG)
        .inner_margin(Margin::symmetric(8, 5))
        .stroke(Stroke::new(1.0, BORDER))
}

pub fn tray_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(TRAY_BG)
        .inner_margin(Margin::symmetric(8, 8))
        .stroke(Stroke::new(1.0, BORDER))
}
