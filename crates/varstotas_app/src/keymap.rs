//! Keyboard shortcuts: the commands that take one, their default keys, the user's
//! reassignments (`Prefs::shortcuts`: command id → `"Cmd+Shift+Z, Cmd+Y"`, empty = none) and
//! the lookup the keyboard handler, menus, toolbar tips and the guide read.

use crate::app::Action;
use crate::tools::ToolId;
use egui::{Key, Modifiers};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// One key with modifiers (`cmd` = ⌘ on macOS, Ctrl elsewhere).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: Key,
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Shortcut {
    pub const fn plain(key: Key) -> Self {
        Self { key, cmd: false, shift: false, alt: false }
    }
    const fn cmd(key: Key) -> Self {
        Self { key, cmd: true, shift: false, alt: false }
    }
    const fn cmd_shift(key: Key) -> Self {
        Self { key, cmd: true, shift: true, alt: false }
    }
    const fn shift(key: Key) -> Self {
        Self { key, cmd: false, shift: true, alt: false }
    }

    pub fn from_event(key: Key, m: Modifiers) -> Self {
        Self { key, cmd: m.command || m.ctrl || m.mac_cmd, shift: m.shift, alt: m.alt }
    }

    /// Stored form: `Cmd+Shift+Z`.
    pub fn to_store(self) -> String {
        let mut s = String::new();
        for (on, w) in [(self.cmd, "Cmd+"), (self.alt, "Alt+"), (self.shift, "Shift+")] {
            if on {
                s.push_str(w);
            }
        }
        s + self.key.name()
    }

    pub fn from_store(s: &str) -> Option<Self> {
        let mut sc = Self::plain(Key::A);
        let mut parts: Vec<&str> = s.trim().split('+').map(str::trim).collect();
        // `Cmd++` style keys do not occur: every key has a name
        let key = parts.pop()?;
        for p in parts {
            match p.to_ascii_lowercase().as_str() {
                "cmd" | "ctrl" | "command" => sc.cmd = true,
                "alt" | "option" => sc.alt = true,
                "shift" => sc.shift = true,
                _ => return None,
            }
        }
        sc.key = Key::from_name(key)?;
        Some(sc)
    }

    /// What menus and the guide show: `⇧⌘Z` on macOS, `Ctrl+Shift+Z` elsewhere, `Shift+Z`, `L`.
    pub fn label(self) -> String {
        let key = match self.key {
            Key::Space => "Space",
            Key::PageDown => "PgDn",
            Key::PageUp => "PgUp",
            k => k.name(),
        };
        if self.cmd && cfg!(target_os = "macos") {
            let mut s = String::new();
            if self.alt {
                s.push('⌥');
            }
            if self.shift {
                s.push('⇧');
            }
            return s + "⌘" + key;
        }
        let mut s = String::new();
        for (on, w) in [(self.cmd, "Ctrl+"), (self.alt, "Alt+"), (self.shift, "Shift+")] {
            if on {
                s.push_str(w);
            }
        }
        s + key
    }

    /// Keys that cannot be assigned: they belong to typing, the Measurements box and tools.
    pub fn reserved(self) -> Option<&'static str> {
        let typing = !self.cmd && !self.alt;
        match self.key {
            Key::Escape | Key::Enter | Key::Tab | Key::Backspace | Key::Delete => Some("this key is used by every tool"),
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => Some("arrows lock the inference axes"),
            Key::F1 => Some("F1 opens the guide"),
            k if typing && k.name().chars().all(|c| c.is_ascii_digit()) => Some("digits type into Measurements"),
            Key::Minus | Key::Period | Key::Comma | Key::Semicolon | Key::Slash | Key::Quote if typing => {
                Some("this key types into Measurements")
            }
            Key::X | Key::C | Key::V if self.cmd && !self.alt => Some("Cut / Copy / Paste"),
            _ => None,
        }
    }
}

/// A command that has (or can have) a shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCmd {
    New,
    Open,
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    SelectAll,
    SelectNone,
    MakeGroup,
    ZoomExtents,
    ToggleSnapping,
    NextScene,
    PrevScene,
    Tool(ToolId),
}

const COMMANDS: [KeyCmd; 14] = [
    KeyCmd::New,
    KeyCmd::Open,
    KeyCmd::Save,
    KeyCmd::SaveAs,
    KeyCmd::Quit,
    KeyCmd::Undo,
    KeyCmd::Redo,
    KeyCmd::SelectAll,
    KeyCmd::SelectNone,
    KeyCmd::MakeGroup,
    KeyCmd::ZoomExtents,
    KeyCmd::ToggleSnapping,
    KeyCmd::NextScene,
    KeyCmd::PrevScene,
];

impl KeyCmd {
    /// Commands first, then every tool.
    pub fn all() -> Vec<KeyCmd> {
        COMMANDS.into_iter().chain(ToolId::ALL.into_iter().map(KeyCmd::Tool)).collect()
    }

    /// Id in `Prefs::shortcuts`.
    pub fn id(self) -> String {
        match self {
            KeyCmd::Tool(t) => format!("tool:{t:?}"),
            c => format!("{c:?}"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            KeyCmd::New => "New",
            KeyCmd::Open => "Open…",
            KeyCmd::Save => "Save",
            KeyCmd::SaveAs => "Save As…",
            KeyCmd::Quit => "Quit",
            KeyCmd::Undo => "Undo",
            KeyCmd::Redo => "Redo",
            KeyCmd::SelectAll => "Select All",
            KeyCmd::SelectNone => "Select None",
            KeyCmd::MakeGroup => "Make Group",
            KeyCmd::ZoomExtents => "Zoom Extents",
            KeyCmd::ToggleSnapping => "Snapping On / Off",
            KeyCmd::NextScene => "Next Scene",
            KeyCmd::PrevScene => "Previous Scene",
            KeyCmd::Tool(t) => t.name(),
        }
    }

    pub fn defaults(self) -> Vec<Shortcut> {
        let one = |s| vec![s];
        match self {
            KeyCmd::New => one(Shortcut::cmd(Key::N)),
            KeyCmd::Open => one(Shortcut::cmd(Key::O)),
            KeyCmd::Save => one(Shortcut::cmd(Key::S)),
            KeyCmd::SaveAs => one(Shortcut::cmd_shift(Key::S)),
            KeyCmd::Quit => one(Shortcut::cmd(Key::Q)),
            KeyCmd::Undo => one(Shortcut::cmd(Key::Z)),
            KeyCmd::Redo => vec![Shortcut::cmd_shift(Key::Z), Shortcut::cmd(Key::Y)],
            KeyCmd::SelectAll => one(Shortcut::cmd(Key::A)),
            KeyCmd::SelectNone => one(Shortcut::cmd(Key::T)),
            KeyCmd::MakeGroup => one(Shortcut::plain(Key::G)),
            KeyCmd::ZoomExtents => one(Shortcut::shift(Key::Z)),
            KeyCmd::ToggleSnapping => one(Shortcut::shift(Key::S)),
            KeyCmd::NextScene => one(Shortcut::plain(Key::PageDown)),
            KeyCmd::PrevScene => one(Shortcut::plain(Key::PageUp)),
            KeyCmd::Tool(t) => t.shortcut().map(Shortcut::plain).into_iter().collect(),
        }
    }

    pub fn action(self) -> Action {
        match self {
            KeyCmd::New => Action::New,
            KeyCmd::Open => Action::Open,
            KeyCmd::Save => Action::Save,
            KeyCmd::SaveAs => Action::SaveAs,
            KeyCmd::Quit => Action::Quit,
            KeyCmd::Undo => Action::Undo,
            KeyCmd::Redo => Action::Redo,
            KeyCmd::SelectAll => Action::SelectAll,
            KeyCmd::SelectNone => Action::SelectNone,
            KeyCmd::MakeGroup => Action::MakeGroup,
            KeyCmd::ZoomExtents => Action::ZoomExtents,
            KeyCmd::ToggleSnapping => Action::ToggleSnapping,
            KeyCmd::NextScene => Action::NextScene,
            KeyCmd::PrevScene => Action::PrevScene,
            KeyCmd::Tool(t) => Action::SetTool(t),
        }
    }
}

/// The keys of `cmd` with the user's reassignments applied.
pub fn keys_in(map: &BTreeMap<String, String>, cmd: KeyCmd) -> Vec<Shortcut> {
    match map.get(&cmd.id()) {
        Some(s) => s.split(',').filter_map(Shortcut::from_store).collect(),
        None => cmd.defaults(),
    }
}

fn store(keys: &[Shortcut]) -> String {
    keys.iter().map(|k| k.to_store()).collect::<Vec<_>>().join(", ")
}

/// Give `cmd` the single shortcut `sc` (None = no shortcut). The command that had `sc` loses
/// it; its name is returned. Entries equal to the defaults are dropped from the map.
pub fn assign(map: &mut BTreeMap<String, String>, cmd: KeyCmd, sc: Option<Shortcut>) -> Option<&'static str> {
    let mut taken = None;
    if let Some(sc) = sc {
        for other in KeyCmd::all() {
            let keys = keys_in(map, other);
            if other != cmd && keys.contains(&sc) {
                let rest: Vec<Shortcut> = keys.into_iter().filter(|k| *k != sc).collect();
                set(map, other, &rest);
                taken = Some(other.name());
            }
        }
    }
    set(map, cmd, &sc.into_iter().collect::<Vec<_>>());
    taken
}

/// `cmd` back to its default keys; other commands that had one of them lose it.
pub fn reset(map: &mut BTreeMap<String, String>, cmd: KeyCmd) {
    map.remove(&cmd.id());
    for sc in cmd.defaults() {
        for other in KeyCmd::all() {
            let keys = keys_in(map, other);
            if other != cmd && keys.contains(&sc) {
                let rest: Vec<Shortcut> = keys.into_iter().filter(|k| *k != sc).collect();
                set(map, other, &rest);
            }
        }
    }
}

fn set(map: &mut BTreeMap<String, String>, cmd: KeyCmd, keys: &[Shortcut]) {
    if keys == cmd.defaults().as_slice() {
        map.remove(&cmd.id());
    } else {
        map.insert(cmd.id(), store(keys));
    }
}

/// The published reassignments and the key table built from them.
type Published = (BTreeMap<String, String>, Vec<(Shortcut, KeyCmd)>);

thread_local! {
    static MAP: RefCell<Published> = RefCell::new((BTreeMap::new(), table(&BTreeMap::new())));
}

fn table(map: &BTreeMap<String, String>) -> Vec<(Shortcut, KeyCmd)> {
    KeyCmd::all().into_iter().flat_map(|c| keys_in(map, c).into_iter().map(move |k| (k, c))).collect()
}

/// Make `map` the shortcuts the keyboard handler and the labels use.
pub fn publish(map: &BTreeMap<String, String>) {
    MAP.with(|m| {
        if m.borrow().0 != *map {
            *m.borrow_mut() = (map.clone(), table(map));
        }
    });
}

/// The command bound to this key press.
pub fn lookup(sc: Shortcut) -> Option<KeyCmd> {
    MAP.with(|m| m.borrow().1.iter().find(|(k, _)| *k == sc).map(|(_, c)| *c))
}

/// The label of the first shortcut of `cmd` (menus, tips).
pub fn label(cmd: KeyCmd) -> Option<String> {
    MAP.with(|m| m.borrow().1.iter().find(|(_, c)| *c == cmd).map(|(k, _)| k.label()))
}

/// All shortcut labels of `cmd`, joined (`⇧⌘Z, ⌘Y`).
pub fn labels(cmd: KeyCmd) -> String {
    MAP.with(|m| m.borrow().1.iter().filter(|(_, c)| *c == cmd).map(|(k, _)| k.label()).collect::<Vec<_>>().join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_round_trip() {
        for sc in [Shortcut::cmd_shift(Key::Z), Shortcut::plain(Key::Space), Shortcut::shift(Key::PageDown)] {
            assert_eq!(Shortcut::from_store(&sc.to_store()), Some(sc));
        }
        assert_eq!(Shortcut::from_store("Hyper+K"), None);
    }

    #[test]
    fn assign_moves_a_taken_key() {
        let mut map = BTreeMap::new();
        // L belongs to Line: giving it to Rectangle takes it from Line
        let taken = assign(&mut map, KeyCmd::Tool(ToolId::Rectangle), Some(Shortcut::plain(Key::L)));
        assert_eq!(taken, Some("Line"));
        assert!(keys_in(&map, KeyCmd::Tool(ToolId::Line)).is_empty());
        assert_eq!(keys_in(&map, KeyCmd::Tool(ToolId::Rectangle)), vec![Shortcut::plain(Key::L)]);
        // Redo keeps ⌘Y when ⇧⌘Z goes to Save As
        assign(&mut map, KeyCmd::SaveAs, Some(Shortcut::cmd_shift(Key::Z)));
        assert_eq!(keys_in(&map, KeyCmd::Redo), vec![Shortcut::cmd(Key::Y)]);
        // back to the defaults: the entry disappears
        assign(&mut map, KeyCmd::Tool(ToolId::Line), Some(Shortcut::plain(Key::L)));
        assert!(!map.contains_key("tool:Line"));
        publish(&map);
        assert_eq!(lookup(Shortcut::plain(Key::L)), Some(KeyCmd::Tool(ToolId::Line)));
        // Rectangle gave its R away for L and lost L again: it has none now
        assert_eq!(lookup(Shortcut::plain(Key::R)), None);
        assert!(keys_in(&map, KeyCmd::Tool(ToolId::Rectangle)).is_empty());
        reset(&mut map, KeyCmd::Tool(ToolId::Rectangle));
        reset(&mut map, KeyCmd::Redo);
        // Save As gave its ⇧⌘S away for ⇧⌘Z, then lost ⇧⌘Z to Redo
        assert_eq!(map.get("SaveAs").map(String::as_str), Some(""));
        reset(&mut map, KeyCmd::SaveAs);
        assert!(map.is_empty(), "{map:?}");
    }
}
