//! The Realistic style's UI (track R): the Environment tray card, the physical properties of the
//! current material in the Materials card (with a shaded sphere preview), HDRI and map loading.

use super::{color_f32, color32, theme};
use crate::app::{Action, FaceStylePref, VarstotasApp};
use crate::document::PaletteMaterial;
use egui::{Color32, RichText, Sense, Ui, vec2};
use varstotas_kernel::{CustomShader, EFFECT_PRESETS, Effects, Environment, MaterialId, PBR_MAPS, Pbr, SkyKind, Stylize, TextureImage, Tonemap, shader_params};
use std::path::Path;
use varstotas_i18n::tr;

/// Starting points of the material editor: (name, metallic, roughness, transmission, emission).
pub(crate) const PBR_PRESETS: [(&str, f32, f32, f32, f32); 7] = [
    ("Matte", 0.0, 0.9, 0.0, 0.0), // i18n-ignore
    ("Satin", 0.0, 0.5, 0.0, 0.0), // i18n-ignore
    ("Glossy", 0.0, 0.15, 0.0, 0.0), // i18n-ignore
    ("Metal", 1.0, 0.35, 0.0, 0.0), // i18n-ignore
    ("Chrome", 1.0, 0.05, 0.0, 0.0), // i18n-ignore
    ("Glass", 0.0, 0.03, 1.0, 0.0), // i18n-ignore
    ("Light", 0.0, 0.6, 0.0, 5.0), // i18n-ignore
];

/// Keys of the map names, in the order of `PBR_MAPS`.
const MAP_LABELS: [&str; 5] = ["realistic-map-normal", "realistic-map-roughness", "realistic-map-metallic", "realistic-map-occlusion", "realistic-map-emission"];

impl VarstotasApp {
    /// The current paint material if it is in the model.
    fn current_model_material(&self) -> Option<MaterialId> {
        let cur = self.doc.current_material.as_ref()?;
        self.doc.model.materials.iter().find(|(_, m)| m.name == cur.name).map(|(id, _)| id)
    }

    /// Change the physical properties of material `name` (added to the model from the palette
    /// if needed): one undo step.
    pub(crate) fn edit_pbr(&mut self, name: &str, f: impl FnOnce(&mut Pbr)) {
        let pm = match self.doc.model.materials.values().find(|m| m.name == name) {
            Some(m) => PaletteMaterial::of(m),
            None => crate::ui::tray::PALETTE
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(n, c)| PaletteMaterial::color(*n, crate::ui::tray::rgb(*c)))
                .unwrap_or_else(|| PaletteMaterial::color(name, [1.0; 4])),
        };
        let mut f = Some(f);
        self.doc.commit(&tr!("realistic-undo-material"), |m| {
            let id = pm.ensure_in(m);
            let mat = &mut m.materials[id];
            let mut p = mat.pbr_or_default();
            if let Some(f) = f.take() {
                f(&mut p);
            }
            mat.pbr = Some(p.sanitized());
        });
        if let Some(m) = self.doc.model.materials.values().find(|m| m.name == pm.name) {
            self.doc.current_material = Some(PaletteMaterial::of(m));
        }
    }

    /// Load an image file as map `map` of material `mid` (its tile size follows the base
    /// texture, else 1 m). The map's factor becomes 1 so the image decides.
    pub(crate) fn load_pbr_map(&mut self, mid: MaterialId, map: &'static str, path: &Path) {
        let Some(mat) = self.doc.model.materials.get(mid) else { return };
        let width = mat.image.as_ref().map_or(1000.0, |i| i.size[0]);
        let name = mat.name.clone();
        match crate::textures::load_file(path, width) {
            Ok((tex, _)) => self.set_pbr_map(&name, map, Some(tex)),
            Err(e) => self.flash_error(tr!("realistic-cant-use", path = path.display().to_string(), error = e.to_string())),
        }
    }

    pub(crate) fn set_pbr_map(&mut self, name: &str, map: &'static str, tex: Option<TextureImage>) {
        let loaded = tex.is_some();
        self.edit_pbr(name, |p| {
            if let Some(slot) = p.map_mut(map) {
                *slot = tex;
            }
            if loaded {
                match map {
                    "roughness" => p.roughness = 1.0,
                    "metallic" => p.metallic = 1.0,
                    "emission" if p.emission == [0.0; 3] => p.emission = [1.0; 3],
                    _ => {}
                }
            }
        });
    }

    /// Use an HDRI file as the environment (the sky switches to HDRI).
    pub(crate) fn load_hdri(&mut self, path: &Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.flash_error(tr!("realistic-cant-read", path = path.display().to_string(), error = e.to_string()));
                return;
            }
        };
        if let Err(e) = crate::pbr::decode_hdri(&bytes) {
            self.flash_error(tr!("realistic-cant-use", path = path.display().to_string(), error = e.to_string()));
            return;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("environment.hdr").to_string();
        let tex = TextureImage::new(bytes, name, 1000.0);
        self.doc.mutate_untracked(|m| {
            m.hdri = Some(tex);
            m.environment.sky = SkyKind::Hdri;
        });
    }

    pub(crate) fn environment_panel(&mut self, ui: &mut Ui) {
        let mut e = self.doc.model.environment;
        if self.prefs.face_style != FaceStylePref::Realistic {
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr!("realistic-used-by")).color(theme::text_dim()));
                if ui.button(tr!("view-style-realistic")).clicked() {
                    self.prefs.face_style = FaceStylePref::Realistic;
                }
            });
        }
        let mut clear_hdri = false;
        egui::Grid::new("environment").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("realistic-background"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("env_sky")
                    .selected_text(match e.sky {
                        SkyKind::Sky => tr!("realistic-sky-sky"),
                        SkyKind::Color => tr!("realistic-sky-color"),
                        SkyKind::Hdri => tr!("realistic-sky-hdri"),
                        SkyKind::Physical => tr!("realistic-sky-physical"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut e.sky, SkyKind::Sky, tr!("realistic-sky-sky"));
                        ui.selectable_value(&mut e.sky, SkyKind::Color, tr!("realistic-sky-color"));
                        ui.selectable_value(&mut e.sky, SkyKind::Hdri, tr!("realistic-sky-hdri"));
                        ui.selectable_value(&mut e.sky, SkyKind::Physical, tr!("realistic-sky-physical"))
                            .on_hover_text(tr!("realistic-sky-physical-tip"));
                    });
                if e.sky == SkyKind::Color {
                    let mut c = color32([e.color[0], e.color[1], e.color[2], 1.0]);
                    if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                        let f = color_f32(c);
                        e.color = [f[0], f[1], f[2]];
                    }
                }
            });
            ui.end_row();
            if e.sky == SkyKind::Physical {
                ui.label(tr!("realistic-haze"));
                ui.add(egui::Slider::new(&mut e.turbidity, 1.7..=10.0).fixed_decimals(1))
                    .on_hover_text(tr!("realistic-haze-tip"));
                ui.end_row();
            }
            if e.sky == SkyKind::Hdri {
                ui.label(tr!("realistic-sky-hdri"));
                ui.horizontal(|ui| {
                    let name = self.doc.model.hdri.as_ref().map_or_else(|| tr!("realistic-hdri-none"), |h| h.file_name.clone());
                    ui.label(RichText::new(name).small());
                    if ui.button(tr!("realistic-load")).on_hover_text(tr!("realistic-hdri-tip")).clicked() {
                        self.actions.push(Action::PickHdri);
                    }
                    if self.doc.model.hdri.is_some() && ui.button(tr!("realistic-clear")).clicked() {
                        clear_hdri = true;
                    }
                });
                ui.end_row();
            }
            ui.label(tr!("realistic-rotation"));
            ui.add(egui::Slider::new(&mut e.rotation_deg, -180.0..=180.0).suffix("°").fixed_decimals(0));
            ui.end_row();
            ui.label(tr!("realistic-intensity"));
            ui.add(egui::Slider::new(&mut e.intensity, 0.0..=4.0).fixed_decimals(2));
            ui.end_row();
            ui.label(tr!("realistic-sun"));
            ui.add(egui::Slider::new(&mut e.sun, 0.0..=10.0).fixed_decimals(1))
                .on_hover_text(tr!("realistic-sun-tip"));
            ui.end_row();
            ui.label(tr!("realistic-exposure"));
            ui.add(egui::Slider::new(&mut e.exposure, -5.0..=5.0).suffix(" EV").fixed_decimals(1));
            ui.end_row();
            ui.label(tr!("realistic-tone-mapping"));
            egui::ComboBox::from_id_salt("env_tonemap")
                .selected_text(tonemap_name(e.tonemap))
                .show_ui(ui, |ui| {
                    for t in [Tonemap::Aces, Tonemap::Agx, Tonemap::Clamp] {
                        ui.selectable_value(&mut e.tonemap, t, tonemap_name(t));
                    }
                });
            ui.end_row();
        });
        ui.checkbox(&mut e.background, tr!("realistic-show-env"));
        egui::CollapsingHeader::new(tr!("realistic-effects")).id_salt("env_effects").default_open(e.effects.needs_post()).show(ui, |ui| {
            effects_ui(ui, &mut e.effects);
        });
        if ui.small_button(tr!("realistic-reset")).clicked() {
            e = Environment::default();
        }
        if clear_hdri {
            self.doc.mutate_untracked(|m| m.hdri = None);
        }
        if e != self.doc.model.environment {
            self.actions.push(Action::SetEnvironment(e));
        }
    }

    /// Physical properties of the current material (Materials card).
    pub(crate) fn pbr_editor(&mut self, ui: &mut Ui) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        let mid = self.current_model_material();
        let stored = mid.and_then(|id| self.doc.model.materials[id].pbr.clone());
        let base = stored.clone().unwrap_or_default();
        // A slider drag edits a draft (shown in the preview); the model changes when it ends.
        let mut p = match &self.tray.pbr_draft {
            Some((n, d)) if *n == cur.name => d.clone(),
            _ => base.clone(),
        };
        let mut commit = false;
        let mut dragging = false;
        let header = egui::CollapsingHeader::new(tr!("realistic-physical")).id_salt("pbr_editor").default_open(stored.is_some()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let tex = self.pbr_preview(ui.ctx(), &cur, &p);
                let (rect, _) = ui.allocate_exact_size(vec2(84.0, 84.0), Sense::hover());
                ui.painter().image(tex, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                ui.vertical(|ui| {
                    ui.label(RichText::new(tr!("realistic-preset")).small().color(theme::text_dim()));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(3.0, 3.0);
                        for (name, metal, rough, glass, light) in PBR_PRESETS {
                            if ui.small_button(pbr_preset_label(name)).clicked() {
                                p.metallic = metal;
                                p.roughness = rough;
                                p.transmission = glass;
                                if light > 0.0 {
                                    let c = cur.color;
                                    p.emission = [c[0], c[1], c[2]];
                                    p.emission_strength = light;
                                } else {
                                    p.emission = [0.0; 3];
                                }
                                commit = true;
                            }
                        }
                    });
                });
            });
            egui::Grid::new("pbr_values").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                ui.spacing_mut().slider_width = 120.0;
                let mut slider = |ui: &mut Ui, v: &mut f32, r: std::ops::RangeInclusive<f32>, label: String, tip: String| {
                    ui.label(label).on_hover_text(tip);
                    let resp = ui.add(egui::Slider::new(v, r).fixed_decimals(2));
                    dragging |= resp.dragged();
                    commit |= resp.drag_stopped() || (resp.changed() && !resp.dragged());
                    ui.end_row();
                };
                slider(ui, &mut p.metallic, 0.0..=1.0, tr!("realistic-metallic"), tr!("realistic-metallic-tip"));
                slider(ui, &mut p.roughness, 0.0..=1.0, tr!("realistic-roughness"), tr!("realistic-roughness-tip"));
                slider(ui, &mut p.transmission, 0.0..=1.0, tr!("realistic-glass"), tr!("realistic-glass-tip"));
                slider(ui, &mut p.ior, 1.0..=2.5, tr!("realistic-ior"), tr!("realistic-ior-tip"));
                ui.label(tr!("realistic-emission")).on_hover_text(tr!("realistic-emission-tip"));
                ui.horizontal(|ui| {
                    let mut c = color32([p.emission[0], p.emission[1], p.emission[2], 1.0]);
                    if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                        let f = color_f32(c);
                        p.emission = [f[0], f[1], f[2]];
                        commit = true;
                    }
                    let resp = ui.add(egui::DragValue::new(&mut p.emission_strength).range(0.0..=100.0).speed(0.05).prefix("× "));
                    dragging |= resp.dragged();
                    commit |= resp.drag_stopped() || (resp.changed() && !resp.dragged());
                });
                ui.end_row();
            });
            egui::CollapsingHeader::new(tr!("realistic-coat-sheen")).id_salt("pbr_more").default_open(
                p.clearcoat > 0.0 || p.sheen > 0.0 || p.anisotropy != 0.0 || p.anti_tiling > 0.0 || p.bevel > 0.0,
            ).show(ui, |ui| {
                egui::Grid::new("pbr_more_values").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.spacing_mut().slider_width = 120.0;
                    let mut slider = |ui: &mut Ui, v: &mut f32, r: std::ops::RangeInclusive<f32>, id: &str, label: String, tip: String, log: bool| {
                        ui.label(label).on_hover_text(tip);
                        let resp = ui.add(egui::Slider::new(v, r).fixed_decimals(2).logarithmic(log));
                        super::tray_lib::spot(format!("pbr:{id}"), resp.rect.left_center());
                        dragging |= resp.dragged();
                        commit |= resp.drag_stopped() || (resp.changed() && !resp.dragged());
                        ui.end_row();
                    };
                    slider(ui, &mut p.clearcoat, 0.0..=1.0, "Clear coat", tr!("realistic-clearcoat"), tr!("realistic-clearcoat-tip"), false); // i18n-ignore
                    slider(ui, &mut p.clearcoat_roughness, 0.0..=1.0, "Coat rough", tr!("realistic-coat-rough"), tr!("realistic-coat-rough-tip"), false); // i18n-ignore
                    slider(ui, &mut p.sheen, 0.0..=1.0, "Sheen", tr!("realistic-sheen"), tr!("realistic-sheen-tip"), false); // i18n-ignore
                    slider(ui, &mut p.sheen_tint, 0.0..=1.0, "Sheen tint", tr!("realistic-sheen-tint"), tr!("realistic-sheen-tint-tip"), false); // i18n-ignore
                    slider(ui, &mut p.anisotropy, -1.0..=1.0, "Anisotropy", tr!("realistic-anisotropy"), tr!("realistic-anisotropy-tip"), false); // i18n-ignore
                    slider(ui, &mut p.anti_tiling, 0.0..=1.0, "Anti-tiling", tr!("realistic-anti-tiling"), tr!("realistic-anti-tiling-tip"), false); // i18n-ignore
                    slider(ui, &mut p.bevel, 0.0..=100.0, "Bevel, mm", tr!("realistic-bevel"), tr!("realistic-bevel-tip"), true); // i18n-ignore
                });
            });
            ui.horizontal(|ui| {
                ui.label(tr!("realistic-shader")).on_hover_text(tr!("realistic-shader-tip"));
                let shown: String = match &p.shader {
                    None => tr!("realistic-shader-none"),
                    Some(sh) => crate::shaders::preset_name(&sh.source).map_or_else(|| tr!("realistic-shader-custom"), str::to_string),
                };
                egui::ComboBox::from_id_salt("pbr_shader").selected_text(crate::shaders::label(&shown)).show_ui(ui, |ui| {
                    if ui.selectable_label(p.shader.is_none(), tr!("realistic-shader-none")).clicked() {
                        p.shader = None;
                        commit = true;
                    }
                    for (name, src) in crate::shaders::PRESETS {
                        if ui.selectable_label(shown == name, crate::shaders::label(name)).clicked() {
                            p.shader = Some(CustomShader::new(src));
                            commit = true;
                        }
                    }
                });
                if ui.small_button(tr!("realistic-shader-edit")).on_hover_text(tr!("realistic-shader-edit-tip")).clicked() {
                    let code = p.shader.as_ref().map_or_else(|| NEW_SHADER.to_string(), |sh| sh.source.clone());
                    self.tray.shader_edit = Some(ShaderEdit::new(cur.name.clone(), code));
                }
            });
            if let Some(sh) = &mut p.shader {
                let err = self.viewport.shader_errors.iter().find(|(src, _)| *src == sh.source).map(|(_, e)| e.clone());
                if let Some(err) = err {
                    ui.label(RichText::new(err).small().color(ERROR_RED));
                }
                let params = shader_params(&sh.source);
                if !params.is_empty() {
                    egui::Grid::new("pbr_shader_params").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                        ui.spacing_mut().slider_width = 120.0;
                        for (i, q) in params.iter().enumerate() {
                            let mut v = sh.param(i);
                            ui.label(if q.label.is_empty() { q.name.as_str() } else { q.label.as_str() });
                            let resp = ui.add(egui::Slider::new(&mut v, q.min..=q.max).clamping(egui::SliderClamping::Never));
                            if resp.changed() {
                                while sh.params.len() <= i {
                                    let k = sh.params.len();
                                    sh.params.push(sh.param(k));
                                }
                                sh.params[i] = v;
                            }
                            dragging |= resp.dragged();
                            commit |= resp.drag_stopped() || (resp.changed() && !resp.dragged());
                            ui.end_row();
                        }
                    });
                }
            }
            ui.add_space(2.0);
            egui::Grid::new("pbr_maps").num_columns(3).spacing([6.0, 3.0]).show(ui, |ui| {
                for (map, label) in PBR_MAPS.iter().zip(MAP_LABELS) {
                    ui.label(tr(label));
                    let name = p.map(map).map_or("—".to_string(), |t| t.file_name.clone()); // i18n-ignore
                    ui.label(RichText::new(name).small().color(theme::text_dim()));
                    ui.horizontal(|ui| {
                        if ui.small_button(tr!("realistic-load")).clicked() {
                            match mid {
                                Some(id) => self.actions.push(Action::PickPbrMap(id, map)),
                                None => self.flash(tr!("realistic-paint-first")),
                            }
                        }
                        if p.map(map).is_some() && ui.small_button(tr!("realistic-clear")).clicked() {
                            self.set_pbr_map(&cur.name, map, None);
                        }
                    });
                    ui.end_row();
                }
            });
            if map_normal_scale(&p) {
                let resp = ui.add(egui::Slider::new(&mut p.normal_scale, 0.0..=4.0).text(tr!("realistic-normal-strength")).fixed_decimals(2));
                dragging |= resp.dragged();
                commit |= resp.drag_stopped() || (resp.changed() && !resp.dragged());
            }
            if stored.is_some() && ui.small_button(tr!("realistic-remove-physical")).clicked() {
                let name = cur.name.clone();
                self.doc.commit(&tr!("realistic-undo-material"), |m| {
                    if let Some(mat) = m.materials.values_mut().find(|x| x.name == name) {
                        mat.pbr = None;
                    }
                });
                self.tray.pbr_draft = None;
            }
        });
        super::tray_lib::spot("pbr_editor", header.header_response.rect.center());
        if commit {
            self.tray.pbr_draft = None;
            if p != base {
                let q = p.clone();
                self.edit_pbr(&cur.name, move |x| *x = q);
            }
        } else if dragging {
            self.tray.pbr_draft = Some((cur.name.clone(), p));
        }
    }

    /// The shader editor window: changes apply half a second after typing stops when the
    /// shader compiles; errors show under the code.
    pub(crate) fn shader_editor_window(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.tray.shader_edit.take() else { return };
        let mut open = true;
        let mut apply = false;
        egui::Window::new(tr!("realistic-shader-window", material = ed.material.clone()))
            .id(egui::Id::new("shader_editor"))
            .open(&mut open)
            .default_pos(self.viewport.view_rect.min + vec2(60.0, 40.0))
            .default_size(vec2(760.0, 480.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("shader_start").selected_text(tr!("realistic-start-from")).show_ui(ui, |ui| {
                        for (name, src) in crate::shaders::PRESETS.iter().copied().chain([("Checker (new)", NEW_SHADER)]) { // i18n-ignore
                            let name = if src == NEW_SHADER { tr!("realistic-checker-new") } else { crate::shaders::label(name) };
                            if ui.selectable_label(false, name).on_hover_text(tr!("realistic-start-from-tip")).clicked() {
                                ed.code = src.to_string();
                                ed.changed = Some(std::time::Instant::now());
                            }
                        }
                    });
                    ui.toggle_value(&mut ed.reference, tr!("realistic-reference")).on_hover_text(tr!("realistic-reference-tip"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(tr!("apply")).clicked() {
                            apply = true;
                        }
                    });
                });
                let h = (ui.available_height() - 28.0).max(120.0);
                let out = ui
                    .horizontal_top(|ui| {
                        let side = if ed.reference { 230.0 } else { 0.0 };
                        let out = ui.vertical(|ui| {
                            ui.set_width((ui.available_width() - side).max(200.0));
                            ed.editor.show(ui, &mut ed.code, h)
                        });
                        if ed.reference {
                            ui.vertical(|ui| {
                                ui.set_width(side - 8.0);
                                shader_reference(ui, h);
                            });
                        }
                        out.inner
                    })
                    .inner;
                if out.response.changed() {
                    ed.changed = Some(std::time::Instant::now());
                }
                match &ed.error {
                    Some(e) => ui.label(RichText::new(e).color(ERROR_RED)),
                    None => ui.label(RichText::new(tr!("realistic-compiles")).small().color(theme::text_dim())),
                };
            });
        if let Some(t) = ed.changed {
            let left = std::time::Duration::from_millis(500).saturating_sub(t.elapsed());
            if left.is_zero() {
                apply = true;
            } else {
                ctx.request_repaint_after(left);
            }
        }
        if apply {
            ed.changed = None;
            let params: Vec<String> = shader_params(&ed.code).into_iter().map(|q| q.name).collect();
            ed.error = varstotas_render::shaders::check(&varstotas_render::CustomSurface { source: ed.code.clone(), params });
            if ed.error.is_none() {
                let code = ed.code.clone();
                self.edit_pbr(&ed.material, move |p| p.shader = Some(rebased(p.shader.take(), code)));
            }
        }
        if open {
            self.tray.shader_edit = Some(ed);
        }
    }

    /// A sphere shaded with the material on the CPU (cached by the material's values).
    fn pbr_preview(&mut self, ctx: &egui::Context, cur: &PaletteMaterial, p: &Pbr) -> egui::TextureId {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}{:?}{:?}{:?}{}{}{}{}", cur.color, p.metallic, p.roughness, p.transmission, p.emission_strength, p.emission[0], p.emission[1], p.emission[2]).hash(&mut h);
        format!("{:?}{:?}{:?}{:?}", p.clearcoat, p.clearcoat_roughness, p.sheen, p.sheen_tint).hash(&mut h);
        format!("{:?}", self.doc.model.environment).hash(&mut h);
        let key = h.finish();
        if let Some((k, t)) = &self.tray.pbr_preview
            && *k == key
        {
            return t.id();
        }
        let img = preview_sphere(cur.color, p, &self.doc.model.environment, 96);
        let handle = ctx.load_texture("pbr_preview", img, egui::TextureOptions::LINEAR);
        let id = handle.id();
        self.tray.pbr_preview = Some((key, handle));
        id
    }
}

fn map_normal_scale(p: &Pbr) -> bool {
    p.normal_map.is_some()
}

pub(crate) fn tonemap_name(t: Tonemap) -> String {
    tr(match t {
        Tonemap::Aces => "realistic-tonemap-aces",
        Tonemap::Agx => "realistic-tonemap-agx",
        Tonemap::Clamp => "realistic-tonemap-none",
    })
}

/// The shown name of a material preset (`PBR_PRESETS` id).
fn pbr_preset_label(id: &str) -> String {
    tr(match id {
        "Matte" => "realistic-preset-matte", // i18n-ignore
        "Satin" => "realistic-preset-satin", // i18n-ignore
        "Glossy" => "realistic-preset-glossy", // i18n-ignore
        "Metal" => "realistic-preset-metal", // i18n-ignore
        "Chrome" => "realistic-preset-chrome", // i18n-ignore
        "Glass" => "realistic-preset-glass", // i18n-ignore
        _ => "realistic-preset-light",
    })
}

/// The shown name of an effects preset (`EFFECT_PRESETS` id).
fn effect_preset_label(id: &str) -> String {
    tr(match id {
        "off" => "realistic-fx-off",
        "fast" => "realistic-fx-fast",
        "balanced" => "realistic-fx-balanced",
        _ => "realistic-fx-quality",
    })
}

fn srgb_lin(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn lin_srgb(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }
}

/// A `side`² preview: the sphere lit by a studio-like sky and a key light over a checkerboard
/// (shows glass). Same model as the shader, simplified (no maps).
pub(crate) fn preview_sphere(color: [f32; 4], p: &Pbr, env: &Environment, side: usize) -> egui::ColorImage {
    use glam::Vec3;
    let p = p.clone().sanitized();
    let albedo = Vec3::new(srgb_lin(color[0]), srgb_lin(color[1]), srgb_lin(color[2]));
    let alpha = crate::pbr::realistic_alpha(color[3], Some(&p));
    let sky = |d: Vec3| -> Vec3 {
        // bright top, warm horizon, dark floor, a soft box light up-left
        let t = d.y.clamp(-1.0, 1.0);
        let base = if t >= 0.0 { Vec3::new(0.55, 0.6, 0.7).lerp(Vec3::new(0.9, 0.95, 1.0), t) } else { Vec3::new(0.25, 0.23, 0.2) };
        let boxl = Vec3::new(-0.6, 0.7, 0.4).normalize();
        base + Vec3::splat(4.0) * (d.dot(boxl) - 0.9).max(0.0) * 10.0
    };
    let rough = p.roughness.clamp(0.03, 1.0);
    let f0 = Vec3::splat(0.04).lerp(albedo, p.metallic);
    let diffuse = albedo * (1.0 - p.metallic) * (1.0 - p.transmission);
    let tint = albedo / albedo.max_element().max(1e-3);
    let sheen = Vec3::ONE.lerp(tint, p.sheen_tint) * p.sheen * (1.0 - p.metallic);
    let emission = Vec3::new(srgb_lin(p.emission[0]), srgb_lin(p.emission[1]), srgb_lin(p.emission[2])) * p.emission_strength;
    let light = Vec3::new(-0.5, 0.6, 0.62).normalize();
    let sun = env.sun.max(0.0);
    let expo = 0.6 * 2f32.powf(env.exposure);
    let tm = |x: Vec3| -> Vec3 {
        let x = x * expo;
        match env.tonemap {
            Tonemap::Clamp => x.clamp(Vec3::ZERO, Vec3::ONE),
            // ACES fit (AgX is previewed with it too)
            _ => ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(Vec3::ZERO, Vec3::ONE),
        }
    };
    let mut px = Vec::with_capacity(side * side);
    for y in 0..side {
        for x in 0..side {
            let u = (x as f32 + 0.5) / side as f32 * 2.0 - 1.0;
            let w = 1.0 - (y as f32 + 0.5) / side as f32 * 2.0;
            let checker = if ((x / 12) + (y / 12)) % 2 == 0 { 0.92 } else { 0.78 };
            let bg = Vec3::splat(checker);
            let r2 = (u * u + w * w) / 0.88;
            if r2 >= 1.0 {
                px.push(Color32::from_gray((checker * 255.0) as u8));
                continue;
            }
            let n = Vec3::new(u / 0.938, w / 0.938, (1.0 - r2).sqrt()).normalize();
            let v = Vec3::Z;
            let nv = n.dot(v).max(1e-4);
            // split-sum env BRDF (as in the shader)
            let c0 = glam::Vec4::new(-1.0, -0.0275, -0.572, 0.022);
            let c1 = glam::Vec4::new(1.0, 0.0425, 1.04, -0.04);
            let r = rough * c0 + c1;
            let a004 = (r.x * r.x).min((-9.28 * nv).exp2()) * r.x + r.y;
            let ab = glam::Vec2::new(-1.04, 1.04) * a004 + glam::Vec2::new(r.z, r.w);
            let brdf = f0 * ab.x + Vec3::splat(ab.y);
            let refl = (2.0 * nv * n - v).normalize();
            // rough reflections fade toward the average sky
            let env_spec = sky(refl).lerp(Vec3::new(0.6, 0.62, 0.66), rough.sqrt());
            let irr = sky(n).lerp(Vec3::new(0.6, 0.62, 0.66), 0.7);
            let spec_ibl = env_spec * brdf * env.intensity;
            let mut col = diffuse * irr * env.intensity + spec_ibl;
            let nl = n.dot(light);
            if nl > 0.0 && sun > 0.0 {
                let h = (v + light).normalize();
                let nh = n.dot(h).max(0.0);
                let vh = v.dot(h).max(0.0);
                let a2 = (rough * rough).powi(2);
                let dd = nh * nh * (a2 - 1.0) + 1.0;
                let d = a2 / (std::f32::consts::PI * dd * dd);
                let k = (rough + 1.0).powi(2) / 8.0;
                let vis = 1.0 / ((nv * (1.0 - k) + k) * (nl * (1.0 - k) + k));
                let f = f0 + (Vec3::ONE - f0) * (1.0 - vh).powi(5);
                col += (diffuse / std::f32::consts::PI * (Vec3::ONE - f) + d * vis * 0.25 * f) * sun * std::f32::consts::PI * nl;
            }
            // Sheen: a soft rim; the clear coat: a second, sharper reflection on top.
            col += sheen * irr * env.intensity * (0.08 + 0.5 * (1.0 - nv).powi(4));
            if p.clearcoat > 0.0 {
                let fc = p.clearcoat * (0.04 + 0.96 * (1.0 - nv).powi(5));
                let coat = sky(refl).lerp(Vec3::new(0.6, 0.62, 0.66), p.clearcoat_roughness.clamp(0.03, 1.0).sqrt());
                col = col * (1.0 - fc) + coat * fc * env.intensity;
            }
            col += emission;
            let mut c = tm(col);
            if alpha < 0.999 {
                let fr = brdf.y + 0.04;
                let a2 = (alpha + (1.0 - alpha) * fr).clamp(alpha, 1.0);
                c = c * a2 + tm(bg * 0.9) * (1.0 - a2);
            }
            let e = |x: f32| (lin_srgb(x) * 255.0).round() as u8;
            px.push(Color32::from_rgb(e(c.x), e(c.y), e(c.z)));
        }
    }
    egui::ColorImage::new([side, side], px)
}

/// The shader editor's state: the material, its code, when it last changed, the last error.
pub(crate) struct ShaderEdit {
    pub material: String,
    pub code: String,
    pub changed: Option<std::time::Instant>,
    pub error: Option<String>,
    /// The code editor (WGSL highlighting, completion, call hints).
    pub editor: crate::ui::code_editor::ScriptEditor,
    /// The reference column is shown.
    pub reference: bool,
}

impl ShaderEdit {
    pub(crate) fn new(material: String, code: String) -> Self {
        Self { material, code, changed: None, error: None, editor: crate::ui::code_editor::ScriptEditor::wgsl("shader_code"), reference: true }
    }
}

/// The reference column of the shader editor: `Surface`'s fields, the helpers and built-ins.
fn shader_reference(ui: &mut Ui, h: f32) {
    use crate::shader_help as sh;
    egui::ScrollArea::vertical().id_salt("shader_ref").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
        ui.label(RichText::new(tr!("realistic-ref-surface")).strong());
        for (name, ty, doc) in sh::SURFACE {
            ui.label(RichText::new(format!("{name}: {ty}")).monospace().small()).on_hover_text(sh::text(doc));
        }
        ui.add_space(4.0);
        ui.label(RichText::new(tr!("realistic-ref-parameters")).strong());
        ui.label(RichText::new(sh::PARAM_TAG.1).monospace().small()).on_hover_text(sh::text(sh::PARAM_TAG.2));
        ui.add_space(4.0);
        ui.label(RichText::new(tr!("realistic-ref-functions")).strong());
        for (_, sig, doc) in sh::FNS {
            ui.label(RichText::new(*sig).monospace().small()).on_hover_text(sh::text(doc));
        }
    });
}

/// A starting point for a new shader.
const NEW_SHADER: &str = r#"// Checker pattern: `// @param` lines become sliders in the material editor.
// @param size = 200 (10..2000) "Square size (mm)"
// @param contrast = 0.5 (0..1) "Contrast"
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let c = floor(s.pos / p.size);
    let odd = fract((c.x + c.y + c.z) * 0.5) * 2.0;
    o.base = s.base * (1.0 - p.contrast * odd);
    return o;
}
"#;

/// `code` keeping the old shader's parameter values by name.
fn rebased(old: Option<CustomShader>, code: String) -> CustomShader {
    let mut sh = CustomShader::new(code);
    if let Some(old) = old {
        let names = shader_params(&old.source);
        for (i, q) in shader_params(&sh.source).iter().enumerate() {
            if let Some(j) = names.iter().position(|o| o.name == q.name) {
                sh.params[i] = old.param(j);
            }
        }
    }
    sh
}

/// The Effects section of the Environment card: a checkbox per effect, the settings of the
/// ones that are on below it.
fn effects_ui(ui: &mut Ui, x: &mut Effects) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(3.0, 3.0);
        ui.label(RichText::new(tr!("realistic-preset")).small().color(theme::text_dim()));
        for name in EFFECT_PRESETS {
            if ui.small_button(effect_preset_label(name)).clicked()
                && let Some(n) = x.with_preset(name)
            {
                *x = n;
            }
        }
    });
    ui.spacing_mut().slider_width = (ui.available_width() - 150.0).clamp(50.0, 140.0);
    let settings = |ui: &mut Ui, id: &str, rows: &mut dyn FnMut(&mut Ui)| {
        ui.indent(id, |ui| {
            egui::Grid::new(id).num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| rows(ui));
        });
    };
    let slider = |ui: &mut Ui, label: String, v: &mut f32, r: std::ops::RangeInclusive<f32>, log: bool, dec: usize| {
        ui.label(RichText::new(label).small());
        ui.add(egui::Slider::new(v, r).logarithmic(log).fixed_decimals(dec));
        ui.end_row();
    };
    ui.checkbox(&mut x.soft_shadows, tr!("realistic-fx-soft-shadows")).on_hover_text(tr!("realistic-fx-soft-shadows-tip"));
    if x.soft_shadows {
        settings(ui, "fx_soft", &mut |ui| slider(ui, tr!("realistic-fx-sun-size"), &mut x.shadow_softness, 0.1..=5.0, false, 1));
    }
    ui.checkbox(&mut x.ao, tr!("realistic-fx-ao")).on_hover_text(tr!("realistic-fx-ao-tip"));
    if x.ao {
        settings(ui, "fx_ao", &mut |ui| {
            slider(ui, tr!("realistic-fx-radius"), &mut x.ao_radius, 50.0..=5000.0, true, 0);
            slider(ui, tr!("realistic-fx-strength"), &mut x.ao_strength, 0.0..=3.0, false, 2);
        });
    }
    ui.checkbox(&mut x.reflections, tr!("realistic-fx-reflections")).on_hover_text(tr!("realistic-fx-reflections-tip"));
    if x.reflections {
        settings(ui, "fx_ssr", &mut |ui| slider(ui, tr!("realistic-fx-strength"), &mut x.reflection_strength, 0.0..=2.0, false, 2));
    }
    ui.checkbox(&mut x.bloom, tr!("realistic-fx-bloom")).on_hover_text(tr!("realistic-fx-bloom-tip"));
    if x.bloom {
        settings(ui, "fx_bloom", &mut |ui| {
            slider(ui, tr!("realistic-fx-strength"), &mut x.bloom_strength, 0.0..=2.0, false, 2);
            slider(ui, tr!("realistic-fx-threshold"), &mut x.bloom_threshold, 0.0..=8.0, false, 2);
        });
    }
    ui.checkbox(&mut x.dof, tr!("realistic-fx-dof")).on_hover_text(tr!("realistic-fx-dof-tip"));
    if x.dof {
        settings(ui, "fx_dof", &mut |ui| {
            slider(ui, tr!("realistic-fx-blur"), &mut x.aperture, 0.0..=4.0, false, 2);
            ui.label(RichText::new(tr!("realistic-fx-focus")).small()).on_hover_text(tr!("realistic-fx-focus-tip"));
            ui.add(egui::Slider::new(&mut x.focus_distance, 0.0..=100000.0).logarithmic(true).fixed_decimals(0));
            ui.end_row();
        });
    }
    ui.checkbox(&mut x.fog, tr!("realistic-fx-fog")).on_hover_text(tr!("realistic-fx-fog-tip"));
    if x.fog {
        settings(ui, "fx_fog", &mut |ui| {
            slider(ui, tr!("realistic-fx-fog-distance"), &mut x.fog_distance, 1000.0..=1_000_000.0, true, 0);
            ui.label(RichText::new(tr!("realistic-fx-fog-height")).small()).on_hover_text(tr!("realistic-fx-fog-height-tip"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut x.fog_height).speed(50.0).range(0.0..=1_000_000.0));
                let mut c = color32([x.fog_color[0], x.fog_color[1], x.fog_color[2], 1.0]);
                if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                    let f = color_f32(c);
                    x.fog_color = [f[0], f[1], f[2]];
                }
            });
            ui.end_row();
        });
    }
    egui::Grid::new("env_look").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(tr!("realistic-vignette"));
        ui.add(egui::Slider::new(&mut x.vignette, 0.0..=1.0).fixed_decimals(2));
        ui.end_row();
        ui.label(tr!("realistic-style"));
        egui::ComboBox::from_id_salt("env_stylize").selected_text(stylize_name(x.stylize)).show_ui(ui, |ui| {
            for s in Stylize::ALL {
                ui.selectable_value(&mut x.stylize, s, stylize_name(s));
            }
        });
        ui.end_row();
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut x.clay, tr!("realistic-fx-clay")).on_hover_text(tr!("realistic-fx-clay-tip"));
        ui.checkbox(&mut x.high_quality, tr!("realistic-fx-high-quality")).on_hover_text(tr!("realistic-fx-high-quality-tip"));
    });
}

fn stylize_name(s: Stylize) -> String {
    tr(match s {
        Stylize::None => "realistic-stylize-photo",
        Stylize::Pencil => "realistic-stylize-pencil",
        Stylize::Watercolor => "realistic-stylize-watercolor",
        Stylize::Technical => "realistic-stylize-technical",
    })
}

pub(crate) const ERROR_RED: Color32 = Color32::from_rgb(0xC6, 0x28, 0x28);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_shades_materials_differently() {
        let env = Environment::default();
        let rough = preview_sphere([0.8, 0.2, 0.2, 1.0], &Pbr { roughness: 0.9, ..Default::default() }, &env, 32);
        let chrome = preview_sphere([0.8, 0.8, 0.8, 1.0], &Pbr { metallic: 1.0, roughness: 0.05, ..Default::default() }, &env, 32);
        assert_eq!(rough.size, [32, 32]);
        let centre = 16 * 32 + 16;
        assert!(rough.pixels[centre].r() > rough.pixels[centre].g());
        assert_ne!(rough.pixels[centre], chrome.pixels[centre]);
        // corners show the checkerboard
        assert_eq!(rough.pixels[0], chrome.pixels[0]);
    }
}
