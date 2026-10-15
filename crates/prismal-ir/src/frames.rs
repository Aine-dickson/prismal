//! Declared frames (MK-4.2, D-105): before a model is checked, each component read in a
//! frame, `e.in(F).x`, receives a copy of the frame's expressions, so that every analysis of
//! what an expression reads (the order of derived values, what a flow or a guard depends on)
//! sees what the frame reads.

use crate::{FrameDecl, Model};
use serde_json::Value as Json;

/// The model with the frames' expressions copied into the component reads that use them;
/// the model itself when it reads no frame.
pub fn with_reads(model: &Model) -> Model {
    if model.frames.is_empty() {
        return model.clone();
    }
    let Ok(mut json) = serde_json::to_value(model) else { return model.clone() };
    fill(&mut json, &model.frames);
    serde_json::from_value(json).unwrap_or_else(|_| model.clone())
}

fn reads(f: &FrameDecl) -> Vec<Json> {
    std::iter::once(&f.at).chain(std::iter::once(&f.x)).chain(f.y.iter()).filter_map(|e| serde_json::to_value(e).ok()).collect()
}

fn fill(j: &mut Json, frames: &[FrameDecl]) {
    match j {
        Json::Object(map) => {
            if map.contains_key("in") && map.contains_key("frame") && map.contains_key("axis") {
                if let Some(f) = map.get("frame").and_then(Json::as_str).and_then(|id| frames.iter().find(|f| f.id == id)) {
                    map.insert("reads".into(), Json::Array(reads(f)));
                }
            }
            map.values_mut().for_each(|v| fill(v, frames));
        }
        Json::Array(items) => items.iter_mut().for_each(|v| fill(v, frames)),
        _ => {}
    }
}
