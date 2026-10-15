//! JSON in and out of scripts, for the bridge (Python, AI agents): variables are passed as
//! JSON, the last value comes back as JSON. Entities travel as `{"$entity": "<id>", "kind": …}`
//! and vectors as `{"$vec": [x, y, z]}`; an id stays valid as long as the entity lives.

use crate::{Entity, ScriptCtx, ScriptRun, api, run_with};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef, Model};
use rhai::{Array, Dynamic, Map};
use serde_json::{Value, json};
use slotmap::{Key, KeyData};

/// Result of [`eval_json`].
#[derive(Clone, Debug, Default)]
pub struct EvalJson {
    pub run: ScriptRun,
    /// The last value (`null` on an error or `()`).
    pub value: Value,
}

/// Run `src` with `vars` (JSON values) in scope; return the last value as JSON.
pub fn eval_json(model: &mut Model, ctx: &ScriptCtx, src: &str, vars: &serde_json::Map<String, Value>) -> EvalJson {
    let mut vs = Vec::new();
    for (k, v) in vars {
        match from_json(v) {
            Ok(d) => vs.push((k.clone(), d)),
            Err(e) => return EvalJson { run: ScriptRun { error: Some(format!("argument {k}: {e}")), ..Default::default() }, value: Value::Null },
        }
    }
    let (run, value) = run_with(model, ctx, src, vs, to_json);
    EvalJson { run, value: value.unwrap_or(Value::Null) }
}

fn key(k: impl Key) -> u64 {
    k.data().as_ffi()
}

/// Text id of an entity: `<instance>.<instance>…/<v|e|f|g><key>` (root path: empty before `/`).
pub fn entity_id(path: &ContextPath, r: EntityRef) -> String {
    let p = path.iter().map(|i| key(*i).to_string()).collect::<Vec<_>>().join(".");
    let (c, k) = match r {
        EntityRef::Vertex(x) => ('v', key(x)),
        EntityRef::Edge(x) => ('e', key(x)),
        EntityRef::Face(x) => ('f', key(x)),
        EntityRef::Instance(x) => ('g', key(x)),
    };
    format!("{p}/{c}{k}")
}

/// Inverse of [`entity_id`].
pub fn parse_entity_id(s: &str) -> Option<(ContextPath, EntityRef)> {
    let (p, e) = s.split_once('/')?;
    let path = if p.is_empty() {
        Vec::new()
    } else {
        p.split('.').map(|x| x.parse::<u64>().ok().map(|k| KeyData::from_ffi(k).into())).collect::<Option<Vec<_>>>()?
    };
    let mut cs = e.chars();
    let c = cs.next()?;
    let k = KeyData::from_ffi(cs.as_str().parse::<u64>().ok()?);
    let r = match c {
        'v' => EntityRef::Vertex(k.into()),
        'e' => EntityRef::Edge(k.into()),
        'f' => EntityRef::Face(k.into()),
        'g' => EntityRef::Instance(k.into()),
        _ => return None,
    };
    Some((path, r))
}

fn from_json(v: &Value) -> Result<Dynamic, String> {
    Ok(match v {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from_bool(*b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => Dynamic::from_int(i),
            None => Dynamic::from_float(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(a) => Dynamic::from_array(a.iter().map(from_json).collect::<Result<Array, _>>()?),
        Value::Object(o) => {
            if let Some(id) = o.get("$entity") {
                let s = id.as_str().ok_or("$entity must be a string")?;
                let (path, r) = parse_entity_id(s).ok_or_else(|| format!("bad entity id {s}"))?;
                return Ok(Dynamic::from(Entity { path, r }));
            }
            if let Some(Value::Array(a)) = o.get("$vec") {
                let c = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(0.0);
                return Ok(Dynamic::from(DVec3::new(c(0), c(1), c(2))));
            }
            let mut m = Map::new();
            for (k, x) in o {
                m.insert(k.as_str().into(), from_json(x)?);
            }
            Dynamic::from_map(m)
        }
    })
}

fn num(f: f64) -> Value {
    serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null)
}

fn to_json(v: &Dynamic) -> Value {
    if v.is_unit() {
        return Value::Null;
    }
    if let Ok(b) = v.as_bool() {
        return Value::Bool(b);
    }
    if let Ok(i) = v.as_int() {
        return json!(i);
    }
    if let Ok(f) = v.as_float() {
        return num(f);
    }
    if let Some(p) = v.clone().try_cast::<DVec3>() {
        return json!({ "$vec": [num(p.x), num(p.y), num(p.z)] });
    }
    if let Some(e) = v.clone().try_cast::<Entity>() {
        return json!({ "$entity": entity_id(&e.path, e.r), "kind": api::kind_of(&e) });
    }
    if let Some(a) = v.clone().try_cast::<Array>() {
        return Value::Array(a.iter().map(to_json).collect());
    }
    if let Some(m) = v.clone().try_cast::<Map>() {
        return Value::Object(m.iter().map(|(k, x)| (k.to_string(), to_json(x))).collect());
    }
    if v.is_string() {
        return Value::String(v.to_string());
    }
    Value::String(api::show(v))
}
