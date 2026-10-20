//! Modifier keys named the way the platform names them. Hints are written with markers and
//! shown through [`tr`]:
//! - `%COPY%` — the copy / toggle modifier: Option on macOS, Ctrl elsewhere;
//! - `%CMD%` — the command modifier: Cmd on macOS, Ctrl elsewhere;
//! - `%ALT%` — Option on macOS, Alt elsewhere;
//! - `%CMDK%` — the command key as a shortcut prefix: `⌘` on macOS, `Ctrl+` elsewhere.

use std::borrow::Cow;

const MAC: bool = cfg!(target_os = "macos");

fn name(marker: &str, mac: bool) -> &'static str {
    match (marker, mac) {
        ("COPY", true) | ("ALT", true) => "Option",
        ("COPY", false) | ("CMD", false) => "Ctrl",
        ("CMD", true) => "Cmd",
        ("ALT", false) => "Alt",
        ("CMDK", true) => "⌘",
        ("CMDK", false) => "Ctrl+",
        _ => "",
    }
}

/// `s` with its modifier markers named for this platform.
pub fn tr(s: &str) -> Cow<'_, str> {
    tr_for(s, MAC)
}

fn tr_for(s: &str, mac: bool) -> Cow<'_, str> {
    if !s.contains('%') {
        return Cow::Borrowed(s);
    }
    let mut out = s.to_string();
    for m in ["CMDK", "COPY", "CMD", "ALT"] {
        out = out.replace(&format!("%{m}%"), name(m, mac));
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_platform() {
        let s = "%COPY% = copy, %CMD%+Enter runs, %ALT% = sample, `%CMDK%A` selects all";
        assert_eq!(tr_for(s, true), "Option = copy, Cmd+Enter runs, Option = sample, `⌘A` selects all");
        assert_eq!(tr_for(s, false), "Ctrl = copy, Ctrl+Enter runs, Alt = sample, `Ctrl+A` selects all");
        assert_eq!(tr_for("50% off", false), "50% off");
    }
}
