//! File names made from user and web text.

use std::path::PathBuf;

/// File names for things with no usable name.
pub const COMPONENT: &str = "Component"; // i18n-ignore
pub const MATERIAL: &str = "Material"; // i18n-ignore
pub const MODEL: &str = "Model"; // i18n-ignore

/// `s` as a file name (letters, digits and ` -_.()`, the rest `_`); `fallback` when nothing is
/// left.
pub fn safe_name(s: &str, fallback: &str) -> String {
    let n: String = s.chars().map(|c| if c.is_alphanumeric() || " -_.()".contains(c) { c } else { '_' }).collect();
    let n = n.trim().trim_matches('.');
    if n.is_empty() { fallback.into() } else { n.into() }
}

/// `s` as an ASCII cache key (letters, digits, `-` and `_`, the rest `_`).
pub fn file_safe(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

/// An archive or catalogue path `p` as a relative path inside a folder: `None` for `..`, drive
/// letters and empty paths (a leading `/` is dropped).
pub fn safe_rel(p: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = p.split(['/', '\\']).filter(|s| !s.is_empty() && *s != ".").collect();
    (!parts.is_empty() && !parts.contains(&"..") && !parts.iter().any(|s| s.contains(':'))).then(|| parts.iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_stay_inside() {
        assert_eq!(safe_name(" a/b:c ", "X"), "a_b_c");
        assert_eq!(safe_name("..", "X"), "X");
        assert_eq!(safe_rel("textures/a.jpg"), Some(PathBuf::from("textures").join("a.jpg")));
        assert_eq!(safe_rel("../x"), None);
        assert_eq!(safe_rel("/etc/x"), Some(PathBuf::from("etc").join("x")));
        assert_eq!(safe_rel("C:/x"), None);
        assert_eq!(safe_rel("a/C:x"), None);
    }
}
