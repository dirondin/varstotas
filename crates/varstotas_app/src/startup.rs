//! Command-line document opening, independent of OS file association registration.
use std::{ffi::OsString, path::PathBuf};
use varstotas_i18n::tr;

pub fn usage() -> String {
    tr!("cli-usage")
}

pub fn document(args: impl IntoIterator<Item = OsString>) -> Result<Option<PathBuf>, String> {
    let mut path = None;
    let mut literal = false;
    let mut needs_path = false;
    for arg in args {
        if !literal {
            if arg.to_string_lossy().starts_with("-psn_") {
                continue;
            }
            if arg == "--" {
                literal = true;
                continue;
            }
            if arg == "--open" {
                if path.is_some() || needs_path {
                    return Err(tr!("cli-one-document"));
                }
                needs_path = true;
                literal = true;
                continue;
            }
            if arg.to_string_lossy().starts_with('-') {
                return Err(tr!(
                    "cli-unknown-option",
                    option = arg.to_string_lossy().to_string()
                ));
            }
        }
        if path.is_some() {
            return Err(tr!("cli-one-document"));
        }
        path = Some(PathBuf::from(arg));
        needs_path = false;
    }
    if needs_path {
        return Err(tr!("cli-open-path"));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Option<PathBuf>, String> {
        document(args.iter().map(OsString::from))
    }
    #[test]
    fn accepts_paths_with_spaces_unicode_and_explicit_open() {
        for args in [
            vec!["/tmp/мой стол.vrst"],
            vec!["--open", "/tmp/мой стол.vrst"],
        ] {
            assert_eq!(
                parse(&args).unwrap(),
                Some(PathBuf::from("/tmp/мой стол.vrst"))
            );
        }
        assert_eq!(
            parse(&["--", "-table.vrst"]).unwrap(),
            Some(PathBuf::from("-table.vrst"))
        );
        assert_eq!(parse(&["-psn_0_123"]).unwrap(), None);
    }
    #[test]
    fn rejects_ambiguous_or_missing_arguments() {
        for args in [vec!["--open"], vec!["a.vrst", "b.vrst"], vec!["--unknown"]] {
            assert!(parse(&args).is_err());
        }
    }
}
