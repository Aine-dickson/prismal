//! Linking shared definitions into models (D-099).
//!
//! Object types, enumerations and functions written outside models live in the document's
//! libraries, written for any space. Before a model is checked, the definitions it reads,
//! directly or through each other, are copied into it, their points and vectors put in the
//! model's space. Identities are kept, so a model reads a shared definition by the same
//! identity in the authored IR and in the linked one.

use crate::{Document, Library, Model, ANY_SPACE};
use serde_json::Value;
use std::collections::BTreeSet;

/// The document with every model linked; its libraries are kept.
pub fn link(mut d: Document) -> Document {
    if d.libraries.is_empty() {
        return d;
    }
    let libs = d.libraries.clone();
    for m in &mut d.models {
        *m = link_model(m.clone(), &libs);
    }
    d
}

/// One model with the shared definitions it reads.
pub fn link_model(mut m: Model, libs: &[Library]) -> Model {
    // Every identity named anywhere in a value: the strings it holds.
    fn strings(v: &Value, out: &mut BTreeSet<String>) {
        match v {
            Value::String(s) => {
                out.insert(s.clone());
            }
            Value::Array(items) => items.iter().for_each(|x| strings(x, out)),
            Value::Object(map) => map.values().for_each(|x| strings(x, out)),
            _ => {}
        }
    }
    let mut named = BTreeSet::new();
    strings(&serde_json::to_value(&m).expect("serializable"), &mut named);
    let (mut enums, mut functions, mut objects) = (vec![], vec![], vec![]);
    let mut taken: BTreeSet<String> = BTreeSet::new();
    loop {
        let mut more = false;
        for l in libs {
            let mut take = |id: &str, v: Value, named: &mut BTreeSet<String>| -> Option<Value> {
                if !named.contains(id) || !taken.insert(id.to_string()) {
                    return None;
                }
                strings(&v, named);
                more = true;
                Some(v)
            };
            for e in &l.enums {
                if let Some(v) = take(&e.id, serde_json::to_value(e).expect("serializable"), &mut named) {
                    enums.push(v);
                }
            }
            for f in &l.functions {
                if let Some(v) = take(&f.id, serde_json::to_value(f).expect("serializable"), &mut named) {
                    functions.push(v);
                }
            }
            for o in &l.objects {
                if let Some(v) = take(&o.id, serde_json::to_value(o).expect("serializable"), &mut named) {
                    objects.push(v);
                }
            }
        }
        if !more {
            break;
        }
    }
    let space = m.default_space.clone().unwrap_or_default();
    let place = |mut v: Value| {
        in_space(&mut v, &space);
        v
    };
    m.enums.extend(enums.into_iter().map(|v| serde_json::from_value(place(v)).expect("an enumeration")));
    m.functions.extend(functions.into_iter().map(|v| serde_json::from_value(place(v)).expect("a function")));
    m.objects.extend(objects.into_iter().map(|v| serde_json::from_value(place(v)).expect("an object type")));
    m
}

/// Puts the points and vectors of a shared definition in `space`: every `"space"` of a type
/// and every `"origin"` of an expression that names `ANY_SPACE`.
fn in_space(v: &mut Value, space: &str) {
    match v {
        Value::Array(items) => items.iter_mut().for_each(|x| in_space(x, space)),
        Value::Object(map) => {
            for (k, x) in map.iter_mut() {
                if (k == "space" || k == "origin" || k == "default_space") && x.as_str() == Some(ANY_SPACE) {
                    *x = Value::String(space.to_string());
                } else {
                    in_space(x, space);
                }
            }
        }
        _ => {}
    }
}
