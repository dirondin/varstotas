//! Help → User Guide: the documentation of the whole application inside it. Topics from the
//! first steps to files and scripts; the tools are shown from their own texts (`ToolId::
//! description` / `instructor`, the same as the Instructor card), so the guide follows the
//! tools. A search goes through every topic and tool. F1 over the model opens the topic of
//! the active tool; the scripting topics link to Help → Script Reference.

use super::script_help::rich;
use super::theme;
use crate::app::{Action, VarstotasApp};
use crate::tools::ToolId;
use egui::{Context, RichText, ScrollArea, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Topic {
    #[default]
    Start,
    Navigation,
    Selecting,
    Drawing,
    Shapes,
    Modifying,
    Inference,
    Measurements,
    Groups,
    Library,
    Materials,
    Organizing,
    Construction,
    Styles,
    Scenes,
    Files,
    Scripts,
    Shortcuts,
    Problems,
}

pub const TOPICS: &[(Topic, &str)] = &[
    (Topic::Start, "Getting started"),
    (Topic::Navigation, "Navigating the view"),
    (Topic::Selecting, "Selecting"),
    (Topic::Drawing, "Drawing"),
    (Topic::Shapes, "3D shapes"),
    (Topic::Modifying, "Modifying"),
    (Topic::Inference, "Inference and snapping"),
    (Topic::Measurements, "Measurements box"),
    (Topic::Groups, "Groups and components"),
    (Topic::Library, "Components library"),
    (Topic::Materials, "Materials and textures"),
    (Topic::Organizing, "Tags, Outliner, Entity Info"),
    (Topic::Construction, "Guides, dimensions, sections"),
    (Topic::Styles, "Styles and rendering"),
    (Topic::Scenes, "Scenes, sheets, drawings"),
    (Topic::Files, "Files, import and export"),
    (Topic::Scripts, "Scripts and extensions"),
    (Topic::Shortcuts, "Keyboard and mouse"),
    (Topic::Problems, "When something goes wrong"),
];

/// A block of a topic: a heading, a paragraph (`code` in backticks), a row (term, meaning),
/// cards of tools, or a button opening the Script Reference on a topic.
enum B {
    H(&'static str),
    P(&'static str),
    Row(&'static str, &'static str),
    Tools(&'static [ToolId]),
    Script(super::script_help::Topic, &'static str),
}

const START: &[B] = &[
    B::P("Varstotas is a 3D modeller: you draw edges, closed edges make faces, faces are pushed and pulled into solids. Everything sticks together until you group it."),
    B::H("The window"),
    B::Row("Menu bar", "File, Edit, View, Camera, Draw, Tools, Extensions, Window, Help"),
    B::Row("Toolbar", "the tools (a tooltip names each one and its key), navigation and Snapping; views and styles are in the viewport's top-right corner"),
    B::Row("Viewport", "the model; the red, green and blue lines are the axes, the ground is under them"),
    B::Row("Side Bar", "cards on the right: Entity Info, Materials, Components, Tags, Outliner, Scenes, Shadows, Environment, Soften Edges, Instructor (Window → Side Bar turns cards on and off)"),
    B::Row("Status bar", "what the active tool expects, messages and errors, the Log button, the Measurements box"),
    B::H("A first model"),
    B::P("1. Press `R` (Rectangle), click a corner on the ground, click the opposite corner — or type `1000,600` and Enter."),
    B::P("2. Press `P` (Push/Pull), click the rectangle, move up and click — or type `750` and Enter."),
    B::P("3. Press `Space` (Select), triple-click the box to select it all, press `G` to make a group."),
    B::P("4. Press `B` (Paint Bucket), pick a material in the Materials card, click faces."),
    B::P("5. `Shift+Z` fits the model in the view; File → Save writes a `.vrst` file."),
    B::H("Help while working"),
    B::P("The Instructor card explains the active tool; F1 over the model opens this guide on it. Help → Demos opens finished models to look at (a furnished house, a material gallery)."),
];

const NAVIGATION: &[B] = &[
    B::H("Mouse and trackpad"),
    B::Row("Middle button drag", "orbit around the point under the cursor"),
    B::Row("Shift + middle drag", "pan"),
    B::Row("Wheel", "zoom toward the cursor"),
    B::Row("Trackpad: two-finger swipe", "orbit; with Shift — pan"),
    B::Row("Trackpad: pinch, or %CMD% + swipe", "zoom"),
    B::P("These work in every tool, so you never need to leave a drawing operation to look around."),
    B::P("Window → Preferences → Navigation changes them: orbit and zoom speed, an inverted wheel, zoom toward the middle of the view instead of the cursor, the middle button panning (Shift orbits), two fingers panning or zooming."),
    B::H("Views"),
    B::P("Camera → Standard Views (and the view drop-down in the viewport's corner): Iso, Top, Bottom, Front, Back, Left, Right. Camera → Parallel Projection / Perspective. `Shift+Z` — Zoom Extents: the whole model in view."),
    B::Tools(&[ToolId::Orbit, ToolId::Pan, ToolId::Zoom, ToolId::PositionCamera, ToolId::LookAround, ToolId::Walk]),
    B::P("Saved views are scenes — see Scenes, sheets, drawings."),
    B::H("The viewport's own controls"),
    B::P("Three icon drop-downs in the viewport's top-right corner hold everything about how it shows the model (hover one to see what it is set to):"),
    B::Row("View", "standard views, Perspective / Parallel Projection, Field of View, Zoom Extents, Viewports 1 / 2 / 4"),
    B::Row("Style", "Shaded … Realistic; X-Ray, Edges, Profiles"),
    B::Row("Show (the eye)", "axes, ground, sky (off = a plain light background), grid of this view; guides, shadows, section planes / cuts / fill of the model (every view)"),
    B::P("Switches keep the menu open; a click outside closes it."),
    B::H("Several viewports"),
    B::P("View → Viewports (or the view drop-down) shows 1, 2 (side by side) or 4 (2 × 2) views of the model. Each has its own camera, style and view switches; new ones start from the top, the front and the right, in parallel projection. A click in a view makes it the active one (blue frame): menus, toolbars and tools act on it, and the click itself already works there."),
    B::P("Presets (View → Viewports, or Viewport Presets in the view drop-down) set up common layouts at once, e.g. Front + Iso or Front, Left, Top + Iso: the flat views in parallel projection, the iso view in perspective."),
];

const SELECTING: &[B] = &[
    B::Tools(&[ToolId::Select]),
    B::P("Edit → Select All (`%CMDK%A`), Select None (`%CMDK%T`). Delete or Backspace erases the selection; Edit → Hide hides it, Unhide All brings everything back."),
    B::P("Right-click the model for the context menu: Erase, Hide, Make Group, Make Component…, Explode, Intersect Faces, Reverse / Orient Faces, Flip Along, Texture → Position, Unfold…, and the commands of section planes and axes."),
    B::P("The Outliner card shows the groups and components as a tree: a click there selects, a double click zooms to the object."),
];

const DRAWING: &[B] = &[
    B::P("Drawing tools work on the face under the cursor, or on the ground / the active work plane. Closed loops of edges become faces; a line across a face splits it."),
    B::P("While a tool runs you can type a value at any time (no need to click the Measurements box) and press Enter — see Measurements box."),
    B::Tools(&[ToolId::Line, ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon, ToolId::ArcCenter, ToolId::Arc, ToolId::Arc3, ToolId::Pie, ToolId::Bezier, ToolId::SmoothCurve, ToolId::Freehand, ToolId::EditCurve]),
];

const SHAPES: &[B] = &[
    B::P("Draw → Shapes builds a whole solid in one operation. The result is a generated component: select it and change its sizes and segments in Entity Info; Explode turns it into plain geometry."),
    B::Tools(&crate::tools::SHAPES),
];

const MODIFYING: &[B] = &[
    B::P("Move, Rotate and Scale work on the selection, or on what you click when nothing is selected. Moving a vertex, edge or face stretches the geometry around it."),
    B::Row("Copy", "Move / Rotate with %COPY% makes a copy"),
    B::Row("Arrays", "after a copy type `5x` (five copies at the same step) or `/5` (five copies between)"),
    B::Tools(&[ToolId::Move, ToolId::Rotate, ToolId::Scale, ToolId::PushPull, ToolId::FollowMe, ToolId::Offset, ToolId::Chamfer, ToolId::Fillet, ToolId::Eraser, ToolId::SolidTools]),
    B::H("Edit menu and context menu"),
    B::Row("Intersect Faces", "with the model, the selection or the context: edges where faces cross"),
    B::Row("Reverse / Orient Faces", "turn the front side (white) outward"),
    B::Row("Flip Along", "mirror the selection along the red, green or blue direction"),
    B::Row("Soften Edges card", "soften / smooth edges by angle"),
    B::Row("Tools → Unfold…", "flatten faces or groups into a cutting pattern (SVG / PDF / DXF)"),
];

const INFERENCE: &[B] = &[
    B::P("While you draw, the cursor snaps to points and directions of the model; a marker and a tooltip say what it found."),
    B::Row("Endpoint, Midpoint", "green / cyan dot"),
    B::Row("On Edge, On Face", "red square / blue diamond"),
    B::Row("Center, Intersection", "of circles and arcs, of edges"),
    B::Row("On Red / Green / Blue Axis", "a line in the axis' colour"),
    B::Row("Parallel / Perpendicular", "magenta line along an edge"),
    B::Row("From Point", "a dotted line from a point along an axis"),
    B::H("Locking"),
    B::P("Hold `Shift` to lock the direction found; the arrow keys lock an axis: → red, ← green, ↑ blue, ↓ parallel / perpendicular to the last edge."),
    B::H("Snapping menu"),
    B::P("The magnet button (and the Snapping menu) turns each inference on or off; `Shift+S` turns snapping off and on altogether. Increments round free moves (10 mm), angles (15°) and scale factors — snapping to geometry and typed values win over them."),
    B::P("View → Grid shows a grid with its own snapping; the step is in Window → Preferences → Drawing, and with Select active typing a length sets it."),
    B::H("Work planes"),
    B::P("The Guide Plane tool (Tools menu) and Window → Work Planes… make a plane to draw on in the air: points in empty space land on it, the axes and the grid follow it."),
];

const MEASUREMENTS: &[B] = &[
    B::P("The Measurements box at the right of the status bar shows the current length, angle or size. Just type while a tool runs and press Enter; Backspace corrects, Esc clears."),
    B::Row("1200, 1.2m, 120cm, 4', 1'6\"", "lengths: the model unit, or a unit you name"),
    B::Row("1000,600", "two values: a rectangle's sides, a box's base"),
    B::Row("45", "an angle (Rotate, Protractor, arcs)"),
    B::Row("24s", "segments of a circle, arc or shape; sides of a polygon"),
    B::Row("5x, /5", "copies in a row after a Move / Rotate copy"),
    B::Row("2, 0.5", "a scale factor"),
    B::P("Typing again right after an operation changes that operation (a different length, more copies)."),
    B::P("The model unit comes from the template (File → New from Template); `,` and `.` both work as the decimal sign."),
];

const GROUPS: &[B] = &[
    B::P("Geometry sticks: edges and faces that touch merge and split each other. To keep a part separate, make it a group or a component."),
    B::Row("Make Group (G)", "a group: one object; copies are independent"),
    B::Row("Make Component…", "a component: copies share one definition — editing one edits all"),
    B::Row("Double-click", "enter a group / component to edit it (the rest dims); Esc or Edit → Close Group leaves"),
    B::Row("Explode", "back to loose geometry"),
    B::Row("Make Unique (right-click, Entity Info)", "a copy of a component gets its own definition (\"Box 2\"): editing it no longer changes the other copies; a generated one gets a copy of its generator (\"Shelf 2\" under In Model)"),
    B::Row("Place from the library again", "the copy in the model is reused while it is unchanged; after you edited it the library's original comes in as a new definition (\"Chair 2\")"),
    B::H("Components with parameters"),
    B::P("Edit → Make Parametric gives a component sizes X / Y / Z (stretch or scale) and a row of copies in Entity Info."),
    B::P("Window → Component Parameters… gives any component parameters: values and formulas that turn, slide, show or paint its parts, an optional script; every copy has its own values (sliders in Entity Info)."),
    B::P("Generators are components built by a script with typed parameters (Window → Generator Editor…). The Draw → Shapes solids are generators too."),
    B::Script(super::script_help::Topic::Component, "Component script in the Script Reference"),
    B::Script(super::script_help::Topic::Generators, "Generators in the Script Reference"),
];

const LIBRARY: &[B] = &[
    B::P("The Components and Materials cards look the same: a top row (Search, Refresh, + to add, the cloud button for the online library, … for the library folder, what to show and the tile size: Small / Large / List), then the sections In Model (the open document), Favourites, Recent, My Library with its folders, and Examples. A section folds with its arrow; a folder's name opens it in the Finder."),
    B::Row("Right-click a tile → Add to Favourites", "the ☆ tile also shows in Favourites at the top (Remove from Favourites takes it out); Recent lists what you placed or picked to paint with lately"),
    B::Row("Right-click a folder name", "New Folder…; for a subfolder also Rename Folder… and Delete Folder (only when it is empty)"),
    B::Row("Right-click a tile → Move to Folder", "move the file into another folder of the library (favourites follow it)"),
    B::Row("Drag a tile into the viewport", "a component is placed where you drop it; a material paints the face under the cursor"),
    B::P("Components lists your library (`~/.varstotas/components`, or a folder you choose with …), the generators, the bundled Examples and the components of the open document."),
    B::Row("Click a tile", "place it: the copy follows the cursor, click to put it down"),
    B::Row("+ → Save Selection to Library…", "write the selection as a new `.vrst` file"),
    B::Row("Right-click a tile", "Place, Set Preview from View, Set Preview Image…, Remove Preview Image, Rename…, Show in Finder, Delete…; for a document component also Select Instances, Save to Library, Parameters…, Delete from Document…"),
    B::H("Pictures"),
    B::P("A tile shows the file's picture when it has one, else a rendered thumbnail. Set Preview from View takes the viewport as it is now — frame the component first. A component placed from the library takes the file's picture with it; File → Preview Image sets the picture of the open document."),
    B::H("3D Library"),
    B::P("The cloud button in the Components card (or Window → 3D Library…) downloads models: Poly Haven, Smithsonian, NASA, Kenney packs, by URL, or from plugin sources. A downloaded model becomes a component in the library with its credit (Entity Info → Source, File → Export → Attributions). Filters by licence, format and size; favourites and the history of downloads."),
    B::P("Each source shows its terms above its models (a plugin's @notice line). A model whose licence limits its use (non-commercial, no derivatives, share-alike, a site's own licence, unknown) gets a warning when it is placed and when the document is exported. Help → Licences… lists Varstotas's licence, the open-source components, file formats and the online libraries with their terms."),
];

const MATERIALS: &[B] = &[
    B::Tools(&[ToolId::Paint, ToolId::TexturePosition]),
    B::P("The Materials card shows the materials of the model (In Model), your library (`~/.varstotas/materials`, or a folder you choose with …) and Examples; right-click a tile for Use, Rename…, Save to Library, Show in Finder, Export…, Delete…. Picking a library material only arms the Paint Bucket: the material joins the model (In Model) with the first face painted. Delete on an In Model tile removes the material from the model, its faces go back to the default (Edit → Undo brings it back). Edit a material there: colour, opacity, a texture image and its size, physical properties for the Realistic style (metal, roughness, glass, emission, maps, coat, sheen)."),
    B::Row("Save to Library / Export… / + → Import…", "share a material between documents (`.vrstmat`)"),
    B::Row("Cloud button (Materials card)", "download PBR scans from Poly Haven and ambientCG, or from plugin sources"),
    B::Row("Shader", "a material can have its own WGSL shader with parameters (the shader editor has its own reference)"),
    B::P("Faces have a front (white) and a back (grey-blue) side; painting a group paints the faces inside it that have no material of their own."),
];

const ORGANIZING: &[B] = &[
    B::Row("Entity Info", "properties of the selection: area, length, volume of solids, tag, material, name, parameters of components and generators"),
    B::Row("Tags", "visibility, colour and name of tags (layers); deleting a tag moves its content to Untagged"),
    B::Row("Outliner", "the tree of groups and components: Zoom To, Edit, Hide, Explode, Move Out of Group, Delete; drag rows to move objects between groups"),
    B::Row("Hide / Unhide All", "Edit menu and context menu"),
    B::Row("Window → Model Info…", "statistics of the document (faces, triangles, components, materials, textures …) and Purge Unused"),
];

const CONSTRUCTION: &[B] = &[
    B::Tools(&[ToolId::Tape, ToolId::Protractor, ToolId::Dimension, ToolId::Text, ToolId::Axes, ToolId::Section, ToolId::GuidePlane]),
    B::P("Guides are selected, moved and deleted like geometry; Edit → Delete Guides removes them all, View → Guides hides them. Tools → 3D Text… makes text as geometry."),
    B::P("View → Section Planes / Section Cuts / Section Fill control how sections are shown; the active cut is saved in scenes."),
];

const STYLES: &[B] = &[
    B::H("View menu (and the viewport's Style drop-down)"),
    B::Row("Shaded, Shaded with Textures, Monochrome", "faces in material colours, with textures, in one colour"),
    B::Row("Hidden Line, Wireframe", "only the edges (hidden ones removed or not)"),
    B::Row("Realistic", "physical materials, lighting from the environment, sun, effects — the Environment card sets sky, HDRI, exposure and effects"),
    B::Row("X-Ray, Edges, Profiles", "see through faces; edges and thick outlines on and off"),
    B::Row("Shadows", "sun shadows; the time and date are in the Shadows card"),
    B::H("Rendering"),
    B::P("Window → Render… renders the view with a path tracer on the GPU: glass, reflections, lights, soft shadows; quality presets, frame size, depth of field, denoising (when Intel OIDN is installed), colour grading; PNG / EXR, or every scene at once."),
    B::P("Lights (point, spot, area) are placed from the Render window; they are selected by clicking their marker and moved and rotated like other objects."),
    B::P("File → Export → 2D Graphic (PNG) saves the view as it is; File → Export → Animation… makes a video or PNG frames of the scenes or a fly-around."),
];

const SCENES: &[B] = &[
    B::Row("Scenes card", "saved views with their style, visible tags and section; tabs above the viewport switch them"),
    B::Row("View → Animation", "Play the scenes; PgDn / PgUp — next / previous scene"),
    B::Row("Window → Sheets…", "layout pages: viewports of scenes at a scale 1:N, texts, a title block, standard views with dimensions, a parts table; multi-page PDF"),
    B::Row("File → Export → 2D Drawing", "the current view as a vector SVG / PDF with profiles, hidden lines, sections and dimensions"),
    B::Row("File → Export → Report", "parts with sizes for cutting, areas by material (CSV)"),
];

const FILES: &[B] = &[
    B::P("Documents are `.vrst` files (textures and pictures are saved inside). File → New from Template picks the units; Window → Preferences sets the default template and the auto-save interval."),
    B::P("Window → Preferences… has sections on the left and a search: General (auto-save, the template of File → New, how many undo steps are kept), Interface (light, dark or system theme, the scale of menus and panels, toolbar icon size), Drawing (grid, snapping and increments — the same switches as the Snapping menu), Libraries (the components and materials folders, Examples, tile size), Downloads (the watched folder), Navigation (mouse and trackpad), Shortcuts (click a shortcut and press the new keys; Backspace removes it, ↺ puts the default back), Render (the render settings of new documents; where ffmpeg and the Open Image Denoise library are) and View (how a new document looks: style, edges, axes, ground, sky, grid, shadows, projection, field of view; Use Current View / Apply to Current View). Changes apply at once; Reset puts a section back to the defaults."),
    B::P("Auto-save writes unsaved changes to `<name>.vrst~autosave` next to the file; after a crash Varstotas offers to recover it."),
    B::Row("Import", "OBJ, STL, DXF, glTF / GLB, 3MF, COLLADA (DAE), FBX, PLY, points from CSV, and formats added by plugins; you can also drop files on the window"),
    B::Row("Export", "OBJ, STL, DXF, glTF (GLB), 3MF, PLY, faces as JSON, PNG, SVG / PDF drawings, sheets, unfold patterns, animation, report, attributions"),
    B::Row("Edit → Cut / Copy / Paste / Paste in Place", "Paste in Place keeps the copied position"),
    B::Row("File → Preview Image", "the picture the library shows for this file"),
];

const SCRIPTS: &[B] = &[
    B::P("Window → Script Console runs Rhai scripts on the model: build, change, select, measure. File → Run Script… runs a `.rhai` (or `.py`) file. A run is one undo step."),
    B::P("Plugins are scripts in `~/.varstotas/plugins`: they become items of the Extensions menu, with dialogs for parameters and picking points; they can add importers, exporters and material / model sources."),
    B::P("Python and AI agents can drive Varstotas through the local bridge (Python client, MCP server)."),
    B::Script(super::script_help::Topic::Basics, "Open the Script Reference"),
    B::Script(super::script_help::Topic::Console, "Console and plugins"),
    B::Script(super::script_help::Topic::Functions, "All functions"),
];

const SHORTCUTS: &[B] = &[
    B::P("Tool keys work when no text field has the focus. Letters typed during an operation go to the Measurements box."),
];

const PROBLEMS: &[B] = &[
    B::Row("Window → Log", "every error and warning of the session — scripts, import / export, libraries, downloads; a red counter in the status bar means new errors"),
    B::Row("Edit → Undo", "every operation can be undone; a failed operation changes nothing"),
    B::Row("Window → Model Info… → Purge Unused", "remove unused components, materials and tags to make the file smaller"),
    B::Row("Faces look grey-blue", "you see their back: select them, right-click → Reverse Faces or Orient Faces"),
    B::Row("Cannot draw where expected", "check Shift / arrow locks, the active work plane (Window → Work Planes…) and whether you are inside a group (Esc leaves it)"),
    B::Row("Something is missing", "Edit → Unhide All, the Tags card (hidden tags), View → Section Planes"),
];

fn page(topic: Topic) -> &'static [B] {
    match topic {
        Topic::Start => START,
        Topic::Navigation => NAVIGATION,
        Topic::Selecting => SELECTING,
        Topic::Drawing => DRAWING,
        Topic::Shapes => SHAPES,
        Topic::Modifying => MODIFYING,
        Topic::Inference => INFERENCE,
        Topic::Measurements => MEASUREMENTS,
        Topic::Groups => GROUPS,
        Topic::Library => LIBRARY,
        Topic::Materials => MATERIALS,
        Topic::Organizing => ORGANIZING,
        Topic::Construction => CONSTRUCTION,
        Topic::Styles => STYLES,
        Topic::Scenes => SCENES,
        Topic::Files => FILES,
        Topic::Scripts => SCRIPTS,
        Topic::Shortcuts => SHORTCUTS,
        Topic::Problems => PROBLEMS,
    }
}

/// The topic that shows `tool`.
pub fn topic_of(tool: ToolId) -> Topic {
    TOPICS
        .iter()
        .find(|(t, _)| page(*t).iter().any(|b| matches!(b, B::Tools(ts) if ts.contains(&tool))))
        .map(|(t, _)| *t)
        .unwrap_or_default()
}

/// Keys other than the tools' own.
fn keys() -> Vec<(String, &'static str)> {
    use crate::keymap::KeyCmd;
    let c = if cfg!(target_os = "macos") { "⌘" } else { "Ctrl+" };
    // reassignable ones (Preferences → Shortcuts), then the fixed keys
    let mut v: Vec<(String, &'static str)> = KeyCmd::all()
        .into_iter()
        .filter(|k| !matches!(k, KeyCmd::Tool(_)))
        .map(|k| (crate::keymap::labels(k), k.name()))
        .filter(|(k, _)| !k.is_empty())
        .collect();
    v.push((format!("{c}X / {c}C / {c}V"), "Cut / Copy / Paste"));
    v.extend(
        [
            ("Delete, Backspace", "erase the selection"),
            ("Esc", "cancel the operation; leave the group being edited"),
            ("Shift (hold)", "lock the inference direction"),
            ("→ ← ↑ ↓", "lock the red / green / blue axis, parallel to the last edge"),
            ("Enter", "apply the value typed in the Measurements box"),
            ("F1", "this guide on the active tool; in a script editor — the Script Reference"),
        ]
        .into_iter()
        .map(|(k, w)| (k.to_string(), w)),
    );
    v
}

#[derive(Default)]
pub struct AppHelp {
    pub open: bool,
    pub topic: Topic,
    pub search: String,
    /// Bring the window to the front on the next frame.
    raise: bool,
}

/// What a click in the guide asks the app to do.
enum Ask {
    Tool(ToolId),
    Script(super::script_help::Topic),
    Topic(Topic),
}

/// A term in bold and what it means (`code` spans in both).
fn row(ui: &mut Ui, term: &str, what: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(4.0);
        ui.vertical(|ui| rich(ui, &format!("**{term}** — {what}")));
    });
}

/// A tool's card: icon, name, key, what it does and how (`Use` picks it).
fn tool_card(ui: &mut Ui, id: ToolId, ask: &mut Option<Ask>) {
    egui::Frame::new().fill(theme::tray_bg()).inner_margin(6.0).corner_radius(4.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
            super::icons::paint_icon(ui.painter(), rect, id.icon(), false);
            ui.label(RichText::new(id.name()).strong().size(14.0));
            if let Some(k) = id.shortcut_label() {
                ui.label(RichText::new(k).monospace().small().color(theme::text_dim()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Use").on_hover_text("Pick this tool").clicked() {
                    *ask = Some(Ask::Tool(id));
                }
            });
        });
        ui.label(RichText::new(id.description()).italics());
        rich(ui, &id.instructor());
    });
    ui.add_space(4.0);
}

fn blocks_ui(ui: &mut Ui, blocks: &[B], ask: &mut Option<Ask>) {
    for b in blocks {
        match b {
            B::H(t) => {
                ui.add_space(6.0);
                ui.label(RichText::new(*t).strong().size(15.0));
            }
            B::P(t) => rich(ui, &crate::keys::tr(t)),
            B::Row(a, b) => row(ui, &crate::keys::tr(a), &crate::keys::tr(b)),
            B::Tools(ids) => {
                ui.add_space(2.0);
                for id in *ids {
                    tool_card(ui, *id, ask);
                }
            }
            B::Script(t, label) => {
                if ui.link(format!("→ {label}")).clicked() {
                    *ask = Some(Ask::Script(*t));
                }
            }
        }
        ui.add_space(3.0);
    }
}

fn topic_ui(ui: &mut Ui, topic: Topic, ask: &mut Option<Ask>) {
    ui.label(RichText::new(TOPICS.iter().find(|t| t.0 == topic).map(|t| t.1).unwrap_or_default()).strong().size(18.0));
    ui.add_space(4.0);
    blocks_ui(ui, page(topic), ask);
    if topic == Topic::Shortcuts {
        ui.label(RichText::new("Tools").strong().size(15.0));
        for id in ToolId::ALL {
            if id.shortcut_label().is_some() {
                row(ui, &crate::keymap::labels(crate::keymap::KeyCmd::Tool(id)), id.name());
            }
        }
        ui.add_space(6.0);
        ui.label(RichText::new("Commands").strong().size(15.0));
        for (k, w) in keys() {
            row(ui, &k, w);
        }
        ui.add_space(6.0);
        rich(ui, "Change them in Window → Preferences → Shortcuts. Mouse and trackpad: see Navigating the view. Selection modifiers: see Selecting.");
    }
}

/// The text of a block that a search looks at.
fn block_text(b: &B) -> Option<String> {
    match b {
        B::H(t) | B::P(t) => Some(crate::keys::tr(t).into_owned()),
        B::Row(a, b) => Some(crate::keys::tr(&format!("{a} {b}")).into_owned()),
        B::Script(_, l) => Some(l.to_string()),
        B::Tools(_) => None,
    }
}

/// Tools whose name, description or instructions contain `q` (lower case).
pub fn find_tools(q: &str) -> Vec<ToolId> {
    ToolId::ALL
        .iter()
        .copied()
        .filter(|t| t.name().to_lowercase().contains(q) || t.description().to_lowercase().contains(q) || t.instructor().to_lowercase().contains(q))
        .collect()
}

/// Topic blocks containing `q` (lower case): the topic and the block's index.
fn find_blocks(q: &str) -> Vec<(Topic, usize)> {
    let mut v = Vec::new();
    for (t, name) in TOPICS {
        if name.to_lowercase().contains(q) {
            v.push((*t, usize::MAX));
        }
        for (i, b) in page(*t).iter().enumerate() {
            if block_text(b).is_some_and(|s| s.to_lowercase().contains(q)) {
                v.push((*t, i));
            }
        }
    }
    v
}

fn search_ui(ui: &mut Ui, q: &str, ask: &mut Option<Ask>) {
    let tools = find_tools(q);
    let hits = find_blocks(q);
    if tools.is_empty() && hits.is_empty() {
        ui.label(RichText::new("Nothing found").color(theme::text_dim()));
        return;
    }
    if !hits.is_empty() {
        ui.label(RichText::new("In the guide").strong().size(15.0));
        let mut last = None;
        for (t, i) in hits {
            if last != Some(t) {
                ui.add_space(4.0);
                let name = TOPICS.iter().find(|x| x.0 == t).map(|x| x.1).unwrap_or_default();
                if ui.link(RichText::new(name).strong()).clicked() {
                    *ask = Some(Ask::Topic(t));
                }
                last = Some(t);
            }
            if let Some(b) = page(t).get(i) {
                ui.indent(("hit", t as u8, i), |ui| blocks_ui(ui, std::slice::from_ref(b), ask));
            }
        }
    }
    if !tools.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new("Tools").strong().size(15.0));
        for id in tools {
            tool_card(ui, id, ask);
        }
    }
}

impl VarstotasApp {
    /// Open the guide on `topic` (in front).
    pub(crate) fn open_app_help(&mut self, topic: Topic) {
        self.app_help.open = true;
        self.app_help.raise = true;
        self.app_help.topic = topic;
        self.app_help.search.clear();
    }

    /// F1 over the model: the guide on the active tool.
    pub(crate) fn app_help_for_tool(&mut self) {
        let t = topic_of(self.tool_id());
        self.open_app_help(t);
    }

    pub(crate) fn app_help_window(&mut self, ctx: &Context) {
        if !self.app_help.open {
            return;
        }
        let mut open = true;
        let mut ask = None;
        let pos = self.viewport.view_rect.left_top() + egui::vec2(60.0, 30.0);
        let mut win = egui::Window::new("User Guide").id(egui::Id::new("app_help")).open(&mut open).default_pos(pos).default_size([700.0, 640.0]).resizable(true);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            let h = &mut self.app_help;
            ui.horizontal(|ui| {
                ui.label("Search");
                let r = ui.add(egui::TextEdit::singleline(&mut h.search).hint_text("tool, command or word").desired_width(240.0));
                if !h.search.is_empty() && ui.small_button("×").clicked() {
                    h.search.clear();
                    r.request_focus();
                }
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(190.0);
                    ScrollArea::vertical().id_salt("app_help_topics").auto_shrink([true, false]).show(ui, |ui| {
                        for (t, name) in TOPICS {
                            let sel = h.search.is_empty() && h.topic == *t;
                            if ui.add(egui::Button::selectable(sel, *name)).clicked() {
                                h.topic = *t;
                                h.search.clear();
                            }
                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ScrollArea::vertical().id_salt(("app_help", h.topic as u8, h.search.is_empty())).auto_shrink([false, false]).show(ui, |ui| {
                        let q = h.search.trim().to_lowercase();
                        if q.is_empty() {
                            topic_ui(ui, h.topic, &mut ask);
                        } else {
                            search_ui(ui, &q, &mut ask);
                        }
                    });
                });
            });
        });
        self.app_help.open = open;
        if std::mem::take(&mut self.app_help.raise) {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, egui::Id::new("app_help")));
        }
        match ask {
            Some(Ask::Tool(id)) => self.actions.push(Action::SetTool(id)),
            Some(Ask::Script(t)) => self.open_script_help(t, None),
            Some(Ask::Topic(t)) => self.open_app_help(t),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool is shown in exactly one topic (F1 finds it there).
    #[test]
    fn every_tool_is_in_one_topic() {
        for id in ToolId::ALL {
            let n = TOPICS.iter().filter(|(t, _)| page(*t).iter().any(|b| matches!(b, B::Tools(ts) if ts.contains(&id)))).count();
            assert_eq!(n, 1, "{} is in {n} topics", id.name());
        }
        assert_eq!(topic_of(ToolId::PushPull), Topic::Modifying);
        assert_eq!(topic_of(ToolId::Sphere), Topic::Shapes);
    }

    #[test]
    fn search_finds_topics_and_tools() {
        assert!(find_tools("push").contains(&ToolId::PushPull));
        assert!(find_blocks("autosave").iter().any(|h| h.0 == Topic::Files));
        assert!(find_blocks("zoom extents").iter().any(|h| h.0 == Topic::Navigation));
    }
}
