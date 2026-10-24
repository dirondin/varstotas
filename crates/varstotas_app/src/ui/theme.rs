//! Light, friendly classic look — and its dark twin (Preferences → Interface).

use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Color32, Context, CornerRadius, Frame, Margin, Stroke, Theme, Visuals};
use serde::{Deserialize, Serialize};

pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x7D, 0xE1);

static DARK: AtomicBool = AtomicBool::new(false);

/// The dark look is on (set by [`set_look`]).
pub fn dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

/// `light` in the light look, `dark` in the dark one.
pub fn pick(light: Color32, dark_c: Color32) -> Color32 {
    if dark() { dark_c } else { light }
}

pub fn bar_bg() -> Color32 {
    pick(Color32::from_rgb(0xF3, 0xF4, 0xF6), Color32::from_rgb(0x2B, 0x2E, 0x33))
}
pub fn toolbar_bg() -> Color32 {
    pick(Color32::from_rgb(0xEC, 0xEE, 0xF1), Color32::from_rgb(0x30, 0x34, 0x3A))
}
pub fn tray_bg() -> Color32 {
    pick(Color32::from_rgb(0xF7, 0xF8, 0xFA), Color32::from_rgb(0x26, 0x29, 0x2E))
}
pub fn border() -> Color32 {
    pick(Color32::from_rgb(0xD3, 0xD7, 0xDD), Color32::from_rgb(0x48, 0x4E, 0x57))
}
pub fn accent_soft() -> Color32 {
    pick(Color32::from_rgb(0xD6, 0xE6, 0xFA), Color32::from_rgb(0x24, 0x48, 0x78))
}
/// Warnings (a texture too large, …): amber, readable in both looks.
pub fn warning() -> Color32 {
    pick(Color32::from_rgb(0xA0, 0x62, 0x00), Color32::from_rgb(0xE0, 0xA8, 0x40))
}
pub fn text_dim() -> Color32 {
    pick(Color32::from_rgb(0x6B, 0x72, 0x7D), Color32::from_rgb(0x9A, 0xA2, 0xAE))
}
/// Fields, cards, tiles: white in the light look.
pub fn field_bg() -> Color32 {
    pick(Color32::WHITE, Color32::from_rgb(0x1E, 0x21, 0x25))
}
/// Cards of the side bar, the console and editor windows: white in the light look.
pub fn card_bg() -> Color32 {
    pick(Color32::WHITE, Color32::from_rgb(0x2E, 0x32, 0x38))
}
/// Hovered list rows and tiles.
pub fn hover_bg() -> Color32 {
    pick(Color32::from_rgb(0xE6, 0xEE, 0xFA), Color32::from_rgb(0x34, 0x40, 0x52))
}
/// Plain text drawn by hand (labels use the visuals).
pub fn text() -> Color32 {
    pick(Color32::from_gray(50), Color32::from_gray(220))
}

/// Light, dark, or as the system is set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePref {
    #[default]
    Light,
    Dark,
    System,
}

/// System fonts with arrows / ⌘ ⇧ etc., appended as fallbacks (egui's bundled fonts lack them).
const SYMBOL_FONTS: &[&str] = &[
    "/System/Library/Fonts/Apple Symbols.ttf", // i18n-ignore
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf", // i18n-ignore
    "C:\\Windows\\Fonts\\seguisym.ttf", // i18n-ignore
    "C:\\Windows\\Fonts\\segoeui.ttf", // i18n-ignore
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

/// Whether `pref` means the dark look now.
pub fn wants_dark(ctx: &Context, pref: ThemePref) -> bool {
    match pref {
        ThemePref::Light => false,
        ThemePref::Dark => true,
        ThemePref::System => ctx.system_theme() == Some(Theme::Dark),
    }
}

pub fn apply(ctx: &Context) {
    install_symbol_fallback(ctx);
    set_look(ctx, false);
}

/// The light or the dark look (a no-op when it is already on).
pub fn set_look(ctx: &Context, dark_on: bool) {
    let first = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("theme_set"))).is_none();
    if !first && dark() == dark_on {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("theme_set"), true));
    DARK.store(dark_on, Ordering::Relaxed);
    let theme = if dark_on { Theme::Dark } else { Theme::Light };
    ctx.set_theme(if dark_on { egui::ThemePreference::Dark } else { egui::ThemePreference::Light });
    let mut v = if dark_on { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = bar_bg();
    v.window_fill = pick(Color32::WHITE, Color32::from_rgb(0x2A, 0x2D, 0x32));
    v.extreme_bg_color = field_bg();
    v.faint_bg_color = pick(Color32::from_rgb(0xF0, 0xF2, 0xF5), Color32::from_rgb(0x31, 0x35, 0x3B));
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.window_stroke = Stroke::new(1.0, border());
    v.selection.bg_fill = accent_soft();
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = pick(ACCENT, Color32::from_rgb(0x6F, 0xA8, 0xF0));
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border());
    v.widgets.inactive.weak_bg_fill = pick(Color32::from_rgb(0xE9, 0xEC, 0xF0), Color32::from_rgb(0x3A, 0x3F, 0x46));
    v.widgets.inactive.bg_fill = pick(Color32::from_rgb(0xE4, 0xE7, 0xEC), Color32::from_rgb(0x40, 0x45, 0x4D));
    v.widgets.hovered.weak_bg_fill = pick(Color32::from_rgb(0xDD, 0xE3, 0xEB), Color32::from_rgb(0x48, 0x50, 0x5B));
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, pick(Color32::from_rgb(0xB8, 0xC2, 0xD0), Color32::from_rgb(0x6A, 0x74, 0x82)));
    v.widgets.active.weak_bg_fill = accent_soft();
    ctx.set_visuals_of(theme, v);
    ctx.style_mut_of(theme, |s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
        s.spacing.menu_margin = Margin::same(6);
        s.spacing.interact_size.y = 22.0;
    });
}

pub fn bar_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(bar_bg())
        .inner_margin(Margin::symmetric(8, 3))
        .stroke(Stroke::new(1.0, border()))
}

pub fn toolbar_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(toolbar_bg())
        .inner_margin(Margin::symmetric(8, 5))
        .stroke(Stroke::new(1.0, border()))
}

pub fn tray_frame(_ctx: &Context) -> Frame {
    Frame::new()
        .fill(tray_bg())
        .inner_margin(Margin::symmetric(8, 8))
        .stroke(Stroke::new(1.0, border()))
}
