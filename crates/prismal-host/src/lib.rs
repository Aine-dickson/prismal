//! The Prismal host interface (`docs/spec/05-host-interface.md`, D-044, D-045).
//!
//! Any system embeds Prismal through an [`Engine`]: it loads documents from source text or
//! the IR, updates and renames them with identities kept, opens their presentations as
//! instances, drives them with its clock and the learner's inputs, and draws the frame
//! descriptions they answer. The same operations are offered as Rust methods and as one JSON
//! protocol, [`Engine::handle`], which every binding (WebAssembly, and later a C ABI or a
//! process on standard input and output) carries unchanged.

pub mod document;
pub mod input;
pub mod instance;

pub use document::{Content, Diagnostic, Document, Span};
pub use instance::{Instance, PointerEvent};

use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

/// The version of the protocol (HI-6.5).
pub const PROTOCOL: u64 = 1;

/// Representation kinds and timeline actions the implementation supports (HI-6.5).
const KINDS: &[&str] = &[
    "marker", "arrow", "segment", "polyline", "polygon", "circle", "ellipse", "arc", "trace", "function_graph", "series_plot", "axes", "grid", "label",
    "equation", "formula", "table", "slider", "number_input", "toggle", "select", "button", "group", "histogram", "ray",
];
const ACTIONS: &[&str] = &[
    "show", "reveal", "hide", "highlight", "camera", "narrate", "wait", "wait_until", "run", "hold", "seek", "reset", "restart", "branch", "intervene", "request",
    "explore", "sequence",
];

/// What the implementation offers (HI-6.5).
pub fn capabilities() -> Json {
    json!({
        "protocol": PROTOCOL,
        "ir": { "format": prismal_ir::FORMAT, "version": prismal_ir::VERSION },
        "representations": KINDS,
        "actions": ACTIONS,
        "media": ["interactive", "video"],
        // Raw input (HI-4.5, D-047): what the host forwards and the keys the engine uses.
        "input": {
            "semantic": ["set_control", "press", "click", "click_at", "request", "key", "pointer_down", "pointer_move", "pointer_up", "cancel", "undo", "redo"],
            // Values from the host's environment for the model's `input` bindings (D-051).
            "environment": ["set_input"],
            "raw": ["pointer", "wheel", "key_down", "focus", "view_reset"],
            "pointers": ["mouse", "pen", "touch"],
            "keys": ["Tab", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Enter", " ", "Escape"],
        },
        "limits": ["an engine is used from one thread at a time (HI-2.3)", "drags on members of a group are not implemented (D-043)"],
    })
}

/// Documents and open instances, by handle (HI section 2).
#[derive(Default)]
pub struct Engine {
    documents: BTreeMap<String, Document>,
    /// Each instance with the handle of its document.
    instances: BTreeMap<String, (String, Instance)>,
    next: u64,
}

fn err(code: &str, message: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::new(code, message)]
}

impl Engine {
    pub fn new() -> Engine {
        Engine::default()
    }

    fn handle_for(&mut self, prefix: &str) -> String {
        self.next += 1;
        format!("{prefix}{}", self.next)
    }

    /// Loads a document (HI-3.1) and answers its handle.
    pub fn load(&mut self, content: Content) -> Result<String, Vec<Diagnostic>> {
        let d = Document::load(content)?;
        let h = self.handle_for("d");
        self.documents.insert(h.clone(), d);
        Ok(h)
    }

    pub fn document(&self, handle: &str) -> Result<&Document, Vec<Diagnostic>> {
        self.documents.get(handle).ok_or_else(|| err("HI-E02", format!("no document `{handle}`")))
    }

    pub fn document_mut(&mut self, handle: &str) -> Result<&mut Document, Vec<Diagnostic>> {
        self.documents.get_mut(handle).ok_or_else(|| err("HI-E02", format!("no document `{handle}`")))
    }

    /// Opens a presentation of a checked document as an instance (HI-4.1). Answers the
    /// instance's handle and layout.
    pub fn open(&mut self, document: &str, presentation: &str, video: bool) -> Result<(String, Json), Vec<Diagnostic>> {
        let prog = self.document(document)?.program().ok_or_else(|| err("HI-E03", "the document has diagnostics; fix them before opening a presentation"))?;
        let mut inst = Instance::new(prog);
        let layout = inst.open(presentation, video).map_err(from_json)?;
        let h = self.handle_for("i");
        self.instances.insert(h.clone(), (document.to_string(), inst));
        Ok((h, layout))
    }

    pub fn instance(&mut self, handle: &str) -> Result<&mut Instance, Vec<Diagnostic>> {
        self.instances.get_mut(handle).map(|x| &mut x.1).ok_or_else(|| err("HI-E02", format!("no instance `{handle}`")))
    }

    /// Closes a document with its instances, or an instance (HI-2.2).
    pub fn close(&mut self, handle: &str) -> Result<(), Vec<Diagnostic>> {
        if self.documents.remove(handle).is_some() {
            self.instances.retain(|_, (d, _)| d != handle);
            Ok(())
        } else if self.instances.remove(handle).is_some() {
            Ok(())
        } else {
            Err(err("HI-E02", format!("no document or instance `{handle}`")))
        }
    }

    /// The number of open documents and instances (for hosts' checks that nothing leaks).
    pub fn open_handles(&self) -> (usize, usize) {
        (self.documents.len(), self.instances.len())
    }

    /// The JSON protocol (HI-6.2): one request, one response, `{"ok": ...}` or
    /// `{"error": [diagnostic, ...]}`.
    pub fn handle(&mut self, request: &str) -> String {
        let out = match serde_json::from_str::<Json>(request) {
            Ok(r) => match self.request(&r) {
                Ok(v) => json!({ "ok": v }),
                Err(ds) => json!({ "error": ds }),
            },
            Err(e) => json!({ "error": err("HI-E01", format!("the request is not JSON: {e}")) }),
        };
        out.to_string()
    }

    /// [`Engine::handle`] on a parsed request.
    pub fn request(&mut self, r: &Json) -> Result<Json, Vec<Diagnostic>> {
        match r.get("protocol").and_then(Json::as_u64) {
            Some(PROTOCOL) => {}
            Some(v) => return Err(err("HI-E01", format!("protocol {v} is not supported; this engine speaks {PROTOCOL}"))),
            None => return Err(err("HI-E01", "a request carries `protocol`")),
        }
        let op = r.get("op").and_then(Json::as_str).ok_or_else(|| err("HI-E01", "a request carries `op`"))?;
        let s = |k: &str| r.get(k).and_then(Json::as_str).ok_or_else(|| err("HI-E01", format!("`{op}` needs `{k}` (a string)")));
        let n = |k: &str| r.get(k).and_then(Json::as_f64).ok_or_else(|| err("HI-E01", format!("`{op}` needs `{k}` (a number)")));
        let content = || -> Result<Content, Vec<Diagnostic>> {
            match (r.get("text").and_then(Json::as_str), r.get("ir")) {
                (Some(t), _) => Ok(Content::Text(t.to_string())),
                (None, Some(ir)) => serde_json::from_value(ir.clone()).map(Content::Ir).map_err(|e| err("HI-E01", format!("`ir` is not an IR document: {e}"))),
                _ => Err(err("HI-E01", format!("`{op}` needs `text` or `ir`"))),
            }
        };
        let doc_info = |d: &Document| json!({ "catalogue": d.catalogue(), "diagnostics": d.diagnostics() });
        match op {
            "capabilities" => Ok(capabilities()),
            "load" => {
                let h = self.load(content()?)?;
                let mut v = doc_info(self.document(&h)?);
                v["document"] = json!(h);
                Ok(v)
            }
            "update" => {
                let c = content()?;
                let d = self.document_mut(s("document")?)?;
                let removed = d.update(c)?;
                let mut v = doc_info(d);
                v["removed"] = json!(removed);
                Ok(v)
            }
            "rename" => {
                let d = self.document_mut(s("document")?)?;
                d.rename(s("element")?, s("name")?).map_err(|e| vec![e])?;
                Ok(json!({ "diagnostics": d.diagnostics() }))
            }
            "ir" => Ok(serde_json::to_value(self.document(s("document")?)?.ir()).expect("IR serializes")),
            "text" => Ok(json!(self.document(s("document")?)?.text())),
            "diagnostics" => Ok(json!(self.document(s("document")?)?.diagnostics())),
            "catalogue" => Ok(self.document(s("document")?)?.catalogue()),
            "locate" => Ok(json!(self.document(s("document")?)?.locate(s("element")?))),
            "run_cases" => self.document(s("document")?)?.run_cases().map_err(|e| vec![e]),
            "close" => {
                let h = r.get("document").or(r.get("instance")).and_then(Json::as_str).ok_or_else(|| err("HI-E01", "`close` needs `document` or `instance`"))?;
                self.close(h).map(|_| Json::Null)
            }
            "open" => {
                let video = match r.get("medium").and_then(Json::as_str) {
                    None | Some("interactive") => false,
                    Some("video") => true,
                    Some(m) => return Err(err("HI-E01", format!("unknown medium `{m}`"))),
                };
                let (h, layout) = self.open(s("document")?, s("presentation")?, video)?;
                Ok(json!({ "instance": h, "layout": layout }))
            }
            _ => {
                let i = self.instance(s("instance")?)?;
                instance_op(i, op, r, &s, &n)
            }
        }
    }
}

type Field<'a, T> = &'a dyn Fn(&str) -> Result<T, Vec<Diagnostic>>;

/// An operation on an open instance (HI section 4).
fn instance_op(i: &mut Instance, op: &str, r: &Json, s: Field<&str>, n: Field<f64>) -> Result<Json, Vec<Diagnostic>> {
    let lesson = i.mode() == "lesson";
    let only = |want_lesson: bool| {
        if want_lesson != lesson {
            let what = if lesson { "a lesson" } else { "an interactive session" };
            Err(err("HI-E03", format!("`{op}` does not apply to {what}")))
        } else {
            Ok(())
        }
    };
    let time = || r.get("time").and_then(Json::as_f64).unwrap_or(0.0);
    // Raw input to a lesson carries the presentation instant it happened at.
    let lesson_time = || if lesson { n("time") } else { Ok(time()) };
    match op {
        "layout" => Ok(i.layout()),
        "frame" => Ok(i.frame(time(), r.get("dt").and_then(Json::as_f64).unwrap_or(0.0))),
        "observations" => Ok(i.observations()),
        "seek" => {
            only(false)?;
            Ok(i.seek(n("time")?))
        }
        "reset" => {
            only(false)?;
            Ok(i.reset())
        }
        "restart" => {
            only(true)?;
            i.lesson_restart().map_err(from_json)
        }
        "continue" => {
            only(true)?;
            i.lesson_continue(n("time")?).map_err(from_json)
        }
        "set_control" if lesson => i.lesson_set_control(n("time")?, s("rep")?, n("value")?).map_err(from_json),
        "set_control" => Ok(i.set_control(s("rep")?, n("value")?)),
        "set_input" => {
            only(false)?;
            Ok(i.set_input(s("input")?, n("value")?))
        }
        "press" if lesson => i.lesson_press(n("time")?, s("rep")?).map_err(from_json),
        "press" => Ok(i.press(s("rep")?)),
        "click" => {
            only(false)?;
            Ok(i.click(s("rep")?))
        }
        "click_at" => {
            only(false)?;
            Ok(i.click_at(s("view")?, n("x")?, n("y")?))
        }
        "request" => {
            only(false)?;
            Ok(i.request(s("event")?, r.get("payload")))
        }
        "key" => {
            only(false)?;
            Ok(i.key(s("rep")?, s("key")?))
        }
        "pointer_down" => {
            only(false)?;
            Ok(i.pointer_down(s("rep")?, r.get("part").and_then(Json::as_str)))
        }
        "pointer_move" => {
            only(false)?;
            Ok(i.pointer_move(n("x")?, n("y")?))
        }
        "pointer_up" => {
            only(false)?;
            Ok(i.pointer_up())
        }
        "pointer" => {
            let e = PointerEvent {
                phase: s("phase")?,
                view: s("view")?,
                x: n("x")?,
                y: n("y")?,
                size: drawn_size(r),
                pointer: r.get("pointer").and_then(Json::as_str).unwrap_or("mouse"),
                id: r.get("id").and_then(Json::as_u64),
                time: lesson_time()?,
            };
            Ok(i.pointer(&e))
        }
        "wheel" => Ok(i.wheel(s("view")?, n("x")?, n("y")?, drawn_size(r), n("delta")?, lesson_time()?)),
        "key_down" => Ok(i.key_down(s("key")?, r.get("shift").and_then(Json::as_bool).unwrap_or(false), lesson_time()?)),
        "focus" => Ok(i.set_focus(r.get("rep").and_then(Json::as_str), lesson_time()?)),
        "view_reset" => Ok(i.view_reset(s("view")?)),
        "cancel" | "undo" | "redo" => {
            only(false)?;
            match op {
                "cancel" => i.cancel(),
                "undo" => i.undo(),
                _ => i.redo(),
            }
            Ok(i.session())
        }
        _ => Err(err("HI-E01", format!("unknown operation `{op}`"))),
    }
}

/// The size a raw input says the host drew its view at (`width`, `height` in pixels).
fn drawn_size(r: &Json) -> Option<[f64; 2]> {
    Some([r.get("width")?.as_f64()?, r.get("height")?.as_f64()?])
}

/// Diagnostics the instance answers as JSON (`[{code, message, element}]` or a message).
fn from_json(v: Json) -> Vec<Diagnostic> {
    match v {
        Json::Array(items) => items
            .into_iter()
            .map(|d| Diagnostic {
                code: d["code"].as_str().unwrap_or("HI-E03").to_string(),
                message: d["message"].as_str().unwrap_or_default().to_string(),
                element: d["element"].as_str().map(str::to_string),
                span: None,
            })
            .collect(),
        Json::String(m) => err("HI-E03", m),
        other => err("HI-E03", other.to_string()),
    }
}
