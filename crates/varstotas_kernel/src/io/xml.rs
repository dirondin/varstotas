//! A small XML tree (on quick-xml) for the 3MF and COLLADA importers: element names and
//! attribute keys without their namespace prefixes.

use super::IoError;
use quick_xml::events::Event;

#[derive(Debug, Default)]
pub(crate) struct El {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<El>,
    pub text: String,
}

fn local(n: &[u8]) -> String {
    let s = String::from_utf8_lossy(n);
    s.rsplit(':').next().unwrap_or(&s).to_string()
}

fn entity(name: &str) -> String {
    match name {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" => "'".into(),
        _ => {
            let code = name.strip_prefix("#x").map(|h| u32::from_str_radix(h, 16).ok()).unwrap_or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()));
            code.and_then(char::from_u32).map(String::from).unwrap_or_default()
        }
    }
}

impl El {
    fn open(e: &quick_xml::events::BytesStart) -> El {
        let attrs = e
            .attributes()
            .flatten()
            .map(|a| (local(a.key.as_ref()), a.normalized_value(quick_xml::XmlVersion::Implicit1_0).map(|v| v.to_string()).unwrap_or_else(|_| String::from_utf8_lossy(&a.value).to_string())))
            .collect();
        El { name: local(e.name().as_ref()), attrs, ..Default::default() }
    }

    /// Parse a document: its root element.
    pub fn parse(text: &str) -> Result<El, IoError> {
        let mut r = quick_xml::Reader::from_str(text);
        let mut stack: Vec<El> = vec![El::default()];
        loop {
            match r.read_event().map_err(|e| IoError::Format(format!("XML: {e}")))? {
                Event::Start(e) => stack.push(El::open(&e)),
                Event::Empty(e) => {
                    let el = El::open(&e);
                    stack.last_mut().unwrap().children.push(el);
                }
                Event::End(_) => {
                    let el = stack.pop().unwrap();
                    match stack.last_mut() {
                        Some(p) => p.children.push(el),
                        None => return Err(IoError::Format("XML: unbalanced tags".into())),
                    }
                }
                Event::Text(t) => {
                    if let Ok(s) = t.decode() {
                        stack.last_mut().unwrap().text.push_str(&s);
                    }
                }
                Event::CData(t) => {
                    stack.last_mut().unwrap().text.push_str(&String::from_utf8_lossy(&t));
                }
                Event::GeneralRef(g) => {
                    let s = entity(&String::from_utf8_lossy(&g));
                    stack.last_mut().unwrap().text.push_str(&s);
                }
                Event::Eof => break,
                _ => {}
            }
            if stack.is_empty() {
                return Err(IoError::Format("XML: unbalanced tags".into()));
            }
        }
        let mut top = stack.swap_remove(0);
        top.children.pop().ok_or_else(|| IoError::Format("XML: no root element".into()))
    }

    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
    }

    pub fn num(&self, k: &str) -> Option<f64> {
        self.attr(k)?.trim().parse().ok()
    }

    pub fn child(&self, name: &str) -> Option<&El> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a El> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// The first element `name` at any depth (self included).
    pub fn find(&self, name: &str) -> Option<&El> {
        if self.name == name {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(name))
    }

    /// Every element `name` at any depth.
    pub fn find_all<'a>(&'a self, name: &str, out: &mut Vec<&'a El>) {
        for c in &self.children {
            if c.name == name {
                out.push(c);
            }
            c.find_all(name, out);
        }
    }

    /// The text as numbers (whitespace separated).
    pub fn numbers(&self) -> Vec<f64> {
        self.text.split_whitespace().filter_map(|s| s.parse().ok()).collect()
    }
}
