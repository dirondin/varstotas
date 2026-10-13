//! Units / VCB (Measurements box) parsing and formatting.
//!
//! Accepted single values (case-insensitive, spaces allowed around parts):
//! - bare numbers in the model unit: `2500`, `-30`, `.5`, `2,5` (decimal comma);
//! - metric suffixes: `250mm`, `25cm`, `2.5m`, `2,5m`, `0.01km`;
//! - imperial: `6"`, `6in`, `10'`, `10ft`, `1'6"`, `1' 6"`, `1'-6"`, `1'6` (inches implied),
//!   fractions `7/16"`, `1 7/16"`, `3/4` (bare, model unit).
//!
//! Lists ([`parse_lengths`]), rule:
//! 1. if the text contains `;` it is the separator (each item may use a decimal comma);
//! 2. otherwise, if it contains `,`, the comma is a separator — **except** for the single
//!    value form `<digits>,<digits><unit>` without any `.` (e.g. `2,5m`), which is one value
//!    with a decimal comma. So `300,200` → two values, `2.5,3.5` → two values,
//!    `2,5m` → one value (2500 mm), `2,5` → two values (2 and 5);
//! 3. otherwise the text is a single value.

use crate::LengthUnit;

const MM_PER_IN: f64 = 25.4;

fn unit_scale(u: LengthUnit) -> f64 {
    match u {
        LengthUnit::Millimeter => 1.0,
        LengthUnit::Centimeter => 10.0,
        LengthUnit::Meter => 1000.0,
        LengthUnit::Inch => MM_PER_IN,
        LengthUnit::Foot => 12.0 * MM_PER_IN,
    }
}

fn suffix_scale(s: &str) -> Option<f64> {
    Some(match s {
        "mm" => 1.0,
        "cm" => 10.0,
        "dm" => 100.0,
        "m" => 1000.0,
        "km" => 1_000_000.0,
        "\"" | "in" | "inch" | "inches" => MM_PER_IN,
        "'" | "ft" | "foot" | "feet" => 12.0 * MM_PER_IN,
        _ => return None,
    })
}

/// Parse an unsigned scalar: `2.5`, `2,5`, `.5`, `7/16`, `1 7/16`.
fn parse_scalar(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.contains('/') {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let (whole, frac) = match parts.as_slice() {
            [f] => (0.0, *f),
            [w, f] => (parse_decimal(w)?, *f),
            _ => return None,
        };
        let (a, b) = frac.split_once('/')?;
        let (a, b) = (parse_decimal(a)?, parse_decimal(b)?);
        if b == 0.0 {
            return None;
        }
        return Some(whole + a / b);
    }
    parse_decimal(s)
}

fn parse_decimal(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    let commas = s.matches(',').count();
    let s = if commas == 1 && !s.contains('.') { s.replace(',', ".") } else if commas == 0 { s.to_string() } else { return None };
    if s == "." {
        return None;
    }
    s.parse::<f64>().ok()
}

/// Parse a length like `2500`, `2.5m`, `250cm`, `10'`, `6"`, `1'6"`, `-30` into mm.
/// Bare numbers use `default_unit`. A single `,` (with no `.`) is a decimal comma.
pub fn parse_length(s: &str, default_unit: LengthUnit) -> Option<f64> {
    let s = s.trim().to_ascii_lowercase();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r.trim_start().to_string()),
        None => (false, s.strip_prefix('+').unwrap_or(&s).trim_start().to_string()),
    };
    if s.is_empty() {
        return None;
    }
    let mm = if let Some(i) = s.find('\'').or_else(|| s.find("ft")) {
        // Feet with optional inches.
        let flen = if s[i..].starts_with("ft") { 2 } else { 1 };
        let feet = parse_scalar(&s[..i])?;
        let mut rest = s[i + flen..].trim();
        rest = rest.strip_prefix('-').unwrap_or(rest).trim();
        let inches = if rest.is_empty() {
            0.0
        } else {
            let r = rest.strip_suffix('"').or_else(|| rest.strip_suffix("in")).unwrap_or(rest);
            parse_scalar(r)?
        };
        feet * 12.0 * MM_PER_IN + inches * MM_PER_IN
    } else {
        let split = s.find(|c: char| c.is_ascii_alphabetic() || c == '"').unwrap_or(s.len());
        let (num, suf) = (s[..split].trim(), s[split..].trim());
        let v = parse_scalar(num)?;
        let k = if suf.is_empty() { unit_scale(default_unit) } else { suffix_scale(suf)? };
        v * k
    };
    if !mm.is_finite() {
        return None;
    }
    Some(if neg { -mm } else { mm })
}

/// Is `s` of the form `<digits>,<digits><unit>` (single value with decimal comma)?
fn is_decimal_comma_with_unit(s: &str) -> bool {
    let s = s.trim().trim_start_matches(['-', '+']);
    let Some((a, b)) = s.split_once(',') else { return false };
    if b.contains(',') || s.contains('.') || a.is_empty() || !a.trim().chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let b = b.trim();
    let digits: String = b.chars().take_while(|c| c.is_ascii_digit()).collect();
    let suf = b[digits.len()..].trim();
    !digits.is_empty() && !suf.is_empty() && suffix_scale(&suf.to_ascii_lowercase()).is_some()
}

/// Parse a list separated by `;` or `,` (see module docs for the exact rule): `300;200`,
/// `300,200`, `2.5m,1.2m`. Every item must parse.
pub fn parse_lengths(s: &str, default_unit: LengthUnit) -> Option<Vec<f64>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let items: Vec<&str> = if s.contains(';') {
        s.split(';').collect()
    } else if s.contains(',') && !is_decimal_comma_with_unit(s) {
        s.split(',').collect()
    } else {
        vec![s]
    };
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        let it = it.trim();
        if it.is_empty() {
            return None;
        }
        // With ',' as list separator the items cannot contain decimal commas anyway.
        out.push(parse_length(it, default_unit)?);
    }
    Some(out)
}

fn trim_num(v: f64, decimals: usize) -> String {
    let s = format!("{:.*}", decimals, v);
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if s == "-0" { "0".into() } else { s }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Inches with a 1/16 fraction, e.g. `2 7/16`.
fn frac_inches(inches: f64) -> (u64, String) {
    let sixteenths = (inches * 16.0).round() as u64;
    let whole = sixteenths / 16;
    let num = sixteenths % 16;
    if num == 0 {
        (whole, whole.to_string())
    } else {
        let g = gcd(num, 16);
        let f = format!("{}/{}", num / g, 16 / g);
        if whole == 0 { (whole, f) } else { (whole, format!("{whole} {f}")) }
    }
}

/// Format mm in the given unit, e.g. `2500 mm`, `2.5 m`, `98 7/16"`, `8' 2 7/16"`.
pub fn format_length(mm: f64, unit: LengthUnit) -> String {
    let neg = mm < 0.0;
    let a = mm.abs();
    let body = match unit {
        LengthUnit::Millimeter => format!("{} mm", trim_num(a, 2)),
        LengthUnit::Centimeter => format!("{} cm", trim_num(a / 10.0, 3)),
        LengthUnit::Meter => format!("{} m", trim_num(a / 1000.0, 4)),
        LengthUnit::Inch => format!("{}\"", frac_inches(a / MM_PER_IN).1),
        LengthUnit::Foot => {
            let total16 = (a / MM_PER_IN * 16.0).round() as u64;
            let feet = total16 / (12 * 16);
            let rem_in = (total16 % (12 * 16)) as f64 / 16.0;
            format!("{}' {}\"", feet, frac_inches(rem_in).1)
        }
    };
    let zero = body.starts_with("0 ") || body == "0\"" || body == "0' 0\"";
    if neg && !zero { format!("-{body}") } else { body }
}

#[cfg(test)]
mod tests {
    use super::*;
    use LengthUnit::*;

    fn close(a: Option<f64>, b: f64) -> bool {
        a.is_some_and(|a| (a - b).abs() < 1e-9)
    }

    #[test]
    fn basic() {
        assert!(close(parse_length("2500", Millimeter), 2500.0));
        assert!(close(parse_length("2.5m", Millimeter), 2500.0));
        assert!(close(parse_length("2,5m", Millimeter), 2500.0));
        assert!(close(parse_length("250cm", Millimeter), 2500.0));
        assert!(close(parse_length("250 cm", Millimeter), 2500.0));
        assert!(close(parse_length("-30", Millimeter), -30.0));
        assert!(close(parse_length("2.5", Meter), 2500.0));
        assert!(close(parse_length(".5m", Millimeter), 500.0));
        assert!(parse_length("abc", Millimeter).is_none());
        assert!(parse_length("", Millimeter).is_none());
        assert!(parse_length("2.5xy", Millimeter).is_none());
        assert!(parse_length("1.2.3", Millimeter).is_none());
    }

    #[test]
    fn imperial() {
        assert!(close(parse_length("6\"", Millimeter), 152.4));
        assert!(close(parse_length("10'", Millimeter), 3048.0));
        assert!(close(parse_length("1'6\"", Millimeter), 457.2));
        assert!(close(parse_length("1' 6\"", Millimeter), 457.2));
        assert!(close(parse_length("1'-6\"", Millimeter), 457.2));
        assert!(close(parse_length("7/16\"", Millimeter), 7.0 / 16.0 * 25.4));
        assert!(close(parse_length("1 7/16\"", Millimeter), 23.0 / 16.0 * 25.4));
        assert!(close(parse_length("3/4", Inch), 0.75 * 25.4));
        assert!(close(parse_length("2ft", Millimeter), 609.6));
        assert!(close(parse_length("12", Foot), 12.0 * 304.8));
        assert!(parse_length("1/0\"", Millimeter).is_none());
    }

    #[test]
    fn lists() {
        assert_eq!(parse_lengths("300;200", Millimeter), Some(vec![300.0, 200.0]));
        assert_eq!(parse_lengths("300,200", Millimeter), Some(vec![300.0, 200.0]));
        assert_eq!(parse_lengths("2.5,3.5", Millimeter), Some(vec![2.5, 3.5]));
        assert_eq!(parse_lengths("2,5;3,5", Millimeter), Some(vec![2.5, 3.5]));
        assert_eq!(parse_lengths("2,5m", Millimeter), Some(vec![2500.0]));
        assert_eq!(parse_lengths("2,5", Millimeter), Some(vec![2.0, 5.0]));
        assert_eq!(parse_lengths("1m, 50cm", Millimeter), Some(vec![1000.0, 500.0]));
        assert_eq!(parse_lengths("1200", Millimeter), Some(vec![1200.0]));
        assert_eq!(parse_lengths("300,", Millimeter), None);
        assert_eq!(parse_lengths("", Millimeter), None);
    }

    #[test]
    fn format() {
        assert_eq!(format_length(2500.0, Millimeter), "2500 mm");
        assert_eq!(format_length(2500.5, Millimeter), "2500.5 mm");
        assert_eq!(format_length(2500.0, Meter), "2.5 m");
        assert_eq!(format_length(2500.0, Centimeter), "250 cm");
        assert_eq!(format_length(-12.0, Millimeter), "-12 mm");
        assert_eq!(format_length(-0.0001, Millimeter), "0 mm");
        assert_eq!(format_length(25.4 * 6.5, Inch), "6 1/2\"");
        let mm = (8.0 * 12.0 + 2.0 + 7.0 / 16.0) * 25.4;
        assert_eq!(format_length(mm, Foot), "8' 2 7/16\"");
        assert_eq!(format_length(304.8, Foot), "1' 0\"");
    }

    #[test]
    fn roundtrip() {
        for v in [1.0, 12.5, 2500.0, 1234.5] {
            for u in [Millimeter, Centimeter, Meter] {
                let s = format_length(v, u);
                assert!(close(parse_length(&s.replace(' ', ""), u), v), "{s}");
            }
        }
    }
}
