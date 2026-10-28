//! UI pieces: menu bar, toolbar, side bar, status bar with the Measurements box, dialogs.

pub mod animation;
pub mod code_editor;
pub mod commands;
pub mod console;
pub mod dialogs;
pub mod downloads;
pub mod extensions;
pub mod feedback;
pub mod gen_editor;
pub mod generators;
pub mod icons;
pub mod library;
pub mod log_window;
pub mod mat_editor;
pub mod mat_maps;
pub mod mat_thumb;
pub mod procedural;
pub mod uv_editor;
pub mod mat_library;
pub mod menu;
pub mod model_info;
pub mod preferences;
pub mod physics;
pub mod printing;
pub mod model_library;
pub mod online_mats;
pub mod realistic;
pub mod render;
pub mod rig_editor;
pub mod licences;
pub mod script_help;
pub mod app_help;
pub mod view_hud;
pub mod sheets;
pub mod statusbar;
pub mod start_screen;
pub mod theme;
pub mod toolbar;
pub mod tray;
pub mod tray_lib;
pub mod unfold;
pub mod cutlist;
pub mod workplanes;

use crate::document::guarded;
use egui::{Color32, Response, Sense, Ui, vec2};
use icons::Icon;
use varstotas_kernel::{LengthUnit, units};

/// Large toolbar button (36×36 with a 24px icon).
pub fn icon_button(ui: &mut Ui, icon: Icon, selected: bool, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if selected {
            p.rect(rect, 7.0, theme::accent_soft(), egui::Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Inside);
        } else if resp.is_pointer_button_down_on() {
            p.rect_filled(rect, 7.0, theme::pick(Color32::from_rgb(0xD2, 0xD9, 0xE3), Color32::from_rgb(0x50, 0x58, 0x64)));
        } else if resp.hovered() {
            p.rect_filled(rect, 7.0, theme::pick(Color32::from_rgb(0xDF, 0xE4, 0xEB), Color32::from_rgb(0x44, 0x4B, 0x55)));
        }
        let icon_size = (size * 0.68).round();
        icons::paint_icon(p, egui::Rect::from_center_size(rect.center(), vec2(icon_size, icon_size)), icon, selected);
    }
    resp
}

/// Small flat icon button for tray rows.
pub fn small_icon_button(ui: &mut Ui, icon: Icon, tooltip: &str) -> Response {
    icon_button(ui, icon, false, 22.0).on_hover_text(tooltip)
}

/// A drag field for a length in model units (mm inside).
pub fn len_drag(v: &mut f64, unit: LengthUnit) -> egui::DragValue<'_> {
    egui::DragValue::new(v)
        .speed(1.0)
        .custom_formatter(move |v, _| units::format_length(v, unit))
        .custom_parser(move |s| units::parse_length(s, unit))
}

pub fn fmt_len(mm: f64, unit: LengthUnit) -> String {
    guarded(|| units::format_length(mm, unit))
        .unwrap_or_else(|_| format!("{:.1} {}", mm / unit.mm(), unit.suffix()))
}

pub fn fmt_area(mm2: f64, unit: LengthUnit) -> String {
    // Large mm/cm areas read better in m² (classic readouts).
    let unit = if matches!(unit, LengthUnit::Millimeter | LengthUnit::Centimeter) && mm2 >= 1.0e5 {
        LengthUnit::Meter
    } else {
        unit
    };
    let f = unit.mm();
    let v = mm2 / (f * f);
    let digits = if v.abs() >= 100.0 { 0 } else { 2 };
    format!("{v:.digits$} {}²", unit.suffix())
}

pub fn fmt_volume(mm3: f64, unit: LengthUnit) -> String {
    // Like `fmt_area`: large mm/cm volumes in m³.
    let unit = if matches!(unit, LengthUnit::Millimeter | LengthUnit::Centimeter) && mm3 >= 1.0e7 {
        LengthUnit::Meter
    } else {
        unit
    };
    let f = unit.mm();
    let v = mm3 / (f * f * f);
    let digits = if v.abs() >= 100.0 { 0 } else { 2 };
    format!("{v:.digits$} {}³", unit.suffix())
}

pub fn color32(c: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

pub fn color_f32(c: Color32) -> [f32; 4] {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
}

/// Paint a colour swatch; returns the response.
pub fn swatch(ui: &mut Ui, color: Color32, size: f32, selected: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let p = ui.painter();
    p.rect_filled(rect, 5.0, color);
    let stroke = if selected {
        egui::Stroke::new(2.5, theme::ACCENT)
    } else if resp.hovered() {
        egui::Stroke::new(1.5, theme::pick(Color32::from_gray(90), Color32::from_gray(190)))
    } else {
        egui::Stroke::new(1.0, Color32::from_black_alpha(50))
    };
    p.rect_stroke(rect, 5.0, stroke, egui::StrokeKind::Inside);
    resp
}

pub mod tutorial;

pub mod printing_tools;
