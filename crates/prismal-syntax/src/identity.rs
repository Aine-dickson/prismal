//! Identities across edits (D-036, IR-1.4, IR-1.7).
//!
//! Lowering gives every element an identity from its declaration path (`Model.speed`,
//! `Model.flow.3`). An element keeps its identity across edits when the compiler matches it
//! to the previous IR:
//!
//! - [`reconcile`] matches each element of a newly lowered document to the previous IR by
//!   its path (by name within its container; flows by target, kind, process and position),
//!   gives it the previous identity, and restores the previous order of every list, new
//!   elements appended (IR-1.7).
//! - [`rename`] is the identity-preserving rename a tool performs (a language server, Mava
//!   Studio): the element keeps its identity and every reference, which is by identity, now
//!   prints the new name.
//! - A rename made by plain text editing is a removal and an addition: [`diff`] reports the
//!   identities that disappeared, so that references to them from elsewhere are reported
//!   broken, never silently rebound (PK-2.3).

use prismal_ir::present::*;
use prismal_ir::*;
use serde_json::Value as Json;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Identities added and removed between two versions of a document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IdentityDiff {
    pub removed: Vec<Id>,
    pub added: Vec<Id>,
}

/// Every identity declared in a document.
pub fn identities(doc: &Document) -> BTreeSet<Id> {
    let mut out = BTreeSet::new();
    fn walk(v: &Json, out: &mut BTreeSet<Id>) {
        match v {
            Json::Object(m) => {
                if let Some(Json::String(id)) = m.get("id") {
                    out.insert(id.clone());
                }
                m.values().for_each(|x| walk(x, out));
            }
            Json::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(&serde_json::to_value(doc).expect("IR serializes"), &mut out);
    out
}

/// The identities removed from `prev` and added in `new`.
pub fn diff(prev: &Document, new: &Document) -> IdentityDiff {
    let (a, b) = (identities(prev), identities(new));
    IdentityDiff { removed: a.difference(&b).cloned().collect(), added: b.difference(&a).cloned().collect() }
}

// ---------------------------------------------------------------- matching

struct Matcher {
    /// New identity to the identity it takes.
    map: HashMap<Id, Id>,
}

impl Matcher {
    fn pair<T>(&mut self, new: &[T], prev: &[T], key: impl Fn(&T) -> String, id: impl Fn(&T) -> &Id) {
        let mut taken = HashSet::new();
        for n in new {
            let k = key(n);
            if let Some(p) = prev.iter().find(|p| key(p) == k && !taken.contains(id(p))) {
                taken.insert(id(p).clone());
                self.map.insert(id(n).clone(), id(p).clone());
            }
        }
    }
}

/// Flows are matched by their target's name, kind and process name, and their position
/// among the flows with the same three.
fn flow_keys(m: &Model) -> Vec<String> {
    let name = |id: &Option<Id>, list: &[Process]| id.as_ref().and_then(|i| list.iter().find(|p| &p.id == i)).map(|p| p.name.clone()).unwrap_or_default();
    let mut seen: HashMap<String, usize> = HashMap::new();
    m.flows
        .iter()
        .map(|f| {
            let target = m.binding(&f.target).map(|b| b.name.clone()).unwrap_or_default();
            let base = format!("{target}|{:?}|{}", f.kind, name(&f.process, &m.processes));
            let n = seen.entry(base.clone()).or_insert(0);
            *n += 1;
            format!("{base}|{n}")
        })
        .collect()
}

fn match_ids(new: &Document, prev: &Document) -> HashMap<Id, Id> {
    let mut mt = Matcher { map: HashMap::new() };
    mt.pair(&new.spaces, &prev.spaces, |s| s.name.clone(), |s| &s.id);
    mt.pair(&new.models, &prev.models, |m| m.name.clone(), |m| &m.id);
    for nm in &new.models {
        let Some(pm) = prev.models.iter().find(|p| p.name == nm.name) else { continue };
        match_body(&mut mt, nm, pm);
    }
    // Shared definitions and modules (D-099), within the library of the same name.
    for nl in &new.libraries {
        let Some(pl) = prev.libraries.iter().find(|p| p.name == nl.name) else { continue };
        mt.pair(&nl.enums, &pl.enums, |e| e.name.clone(), |e| &e.id);
        mt.pair(&nl.functions, &pl.functions, |f| f.name.clone(), |f| &f.id);
        mt.pair(&nl.objects, &pl.objects, |o| o.name.clone(), |o| &o.id);
        for no in &nl.objects {
            if let Some(po) = pl.objects.iter().find(|p| p.name == no.name) {
                match_body(&mut mt, no, po);
            }
        }
    }
    mt.pair(&new.presentations, &prev.presentations, |p| p.name.clone(), |p| &p.id);
    for np in &new.presentations {
        let Some(pp) = prev.presentations.iter().find(|p| p.name == np.name) else { continue };
        mt.pair(&np.observations, &pp.observations, |o| o.name.clone(), |o| &o.id);
        mt.pair(&np.instruments, &pp.instruments, |i| i.name.clone(), |i| &i.id);
        mt.pair(&np.views, &pp.views, |v| v.name.clone(), |v| &v.id);
        for nv in &np.views {
            if let Some(pv) = pp.views.iter().find(|v| v.name == nv.name) {
                pair_reps(&mut mt, &nv.id, &nv.representations, &pv.id, &pv.representations);
            }
        }
        if let (Some(nt), Some(pt)) = (&np.timeline, &pp.timeline) {
            mt.pair(&nt.scenes, &pt.scenes, |s| s.name.clone(), |s| &s.id);
            let nb: Vec<&Beat> = nt.scenes.iter().flat_map(|s| &s.beats).collect();
            let pb: Vec<&Beat> = pt.scenes.iter().flat_map(|s| &s.beats).collect();
            for b in &nb {
                if let Some(p) = pb.iter().find(|p| p.name == b.name) {
                    mt.map.insert(b.id.clone(), p.id.clone());
                    pair_reps(&mut mt, &b.id, &beat_reps(&b.actions), &p.id, &beat_reps(&p.actions));
                }
            }
        }
    }
    mt.pair(&new.runs, &prev.runs, |r| r.name.clone(), |r| &r.id);
    for nr in &new.runs {
        let Some(pr) = prev.runs.iter().find(|r| r.name == nr.name) else { continue };
        for (i, e) in nr.expectations.iter().enumerate() {
            if let Some(p) = pr.expectations.get(i) {
                mt.map.insert(e.id.clone(), p.id.clone());
            }
        }
    }
    mt.map
}

/// The elements of a model or object type, and of its object types and parts (D-055).
fn match_body(mt: &mut Matcher, nm: &Model, pm: &Model) {
    mt.pair(&nm.bindings, &pm.bindings, |b| b.name.clone(), |b| &b.id);
    mt.pair(&nm.processes, &pm.processes, |p| p.name.clone(), |p| &p.id);
    mt.pair(&nm.events, &pm.events, |e| e.name.clone(), |e| &e.id);
    mt.pair(&nm.equations, &pm.equations, |q| q.name.clone(), |q| &q.id);
    mt.pair(&nm.constraints, &pm.constraints, |c| c.name.clone(), |c| &c.id);
    mt.pair(&nm.enums, &pm.enums, |e| e.name.clone(), |e| &e.id);
    mt.pair(&nm.functions, &pm.functions, |f| f.name.clone(), |f| &f.id);
    mt.pair(&nm.parts, &pm.parts, |p| p.name.clone(), |p| &p.id);
    mt.pair(&nm.objects, &pm.objects, |o| o.name.clone(), |o| &o.id);
    mt.pair(&nm.ends, &pm.ends, |e| e.name.clone(), |e| &e.id);
    mt.pair(&nm.grids, &pm.grids, |g| g.name.clone(), |g| &g.id);
    mt.pair(&nm.frames, &pm.frames, |f| f.name.clone(), |f| &f.id);
    for no in &nm.objects {
        if let Some(po) = pm.objects.iter().find(|p| p.name == no.name) {
            match_body(mt, no, po);
        }
    }
    let (nk, pk) = (flow_keys(nm), flow_keys(pm));
    for (i, f) in nm.flows.iter().enumerate() {
        if let Some(j) = pk.iter().position(|k| *k == nk[i]) {
            mt.map.insert(f.id.clone(), pm.flows[j].id.clone());
        }
    }
}

/// Representations are matched within their container by what follows the container's
/// identity: their name, or `kind.n`.
fn pair_reps(mt: &mut Matcher, new_container: &str, new: &[Rep], prev_container: &str, prev: &[Rep]) {
    let suffix = |c: &str, id: &str| id.strip_prefix(c).map(str::to_string).unwrap_or_else(|| id.to_string());
    for r in new {
        let s = suffix(new_container, &r.id);
        if let Some(p) = prev.iter().find(|p| suffix(prev_container, &p.id) == s) {
            mt.map.insert(r.id.clone(), p.id.clone());
            // A group's members are matched within the group (D-043).
            pair_reps(mt, &r.id, &r.members, &p.id, &p.members);
        }
    }
}

/// A representation by identity, among `reps` and the members of groups (D-043).
fn find_rep<'a>(reps: &'a mut [Rep], id: &str) -> Option<&'a mut Rep> {
    for r in reps {
        if r.id == id {
            return Some(r);
        }
        if let Some(m) = find_rep(&mut r.members, id) {
            return Some(m);
        }
    }
    None
}

fn beat_reps(actions: &[Action]) -> Vec<Rep> {
    let mut out = vec![];
    for a in actions {
        match a {
            Action::Show { reps, .. } | Action::Reveal { reps, .. } => out.extend(reps.iter().cloned()),
            Action::Explore { controls, fallback, .. } => {
                out.extend(controls.iter().cloned());
                out.extend(beat_reps(fallback));
            }
            Action::Sequence { actions } | Action::WaitLearner { fallback: actions, .. } => out.extend(beat_reps(actions)),
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------- applying

/// Keys whose string values (or lists of strings) are identities.
const ID_KEYS: &[&str] =
    &["id", "ref", "der", "origin", "space", "target", "binding", "event", "process", "attached_to", "default_space", "model", "presentation", "observation", "beat", "element", "view", "keep", "fn", "enum", "object", "part", "item", "over", "field"];

fn remap(v: &mut Json, map: &HashMap<Id, Id>) {
    match v {
        Json::Object(m) => {
            for (k, x) in m.iter_mut() {
                if ID_KEYS.contains(&k.as_str()) {
                    match x {
                        Json::String(s) => {
                            if let Some(to) = map.get(s.as_str()) {
                                *s = to.clone();
                            }
                        }
                        Json::Array(items) => {
                            for it in items.iter_mut() {
                                if let Json::String(s) = it {
                                    if let Some(to) = map.get(s.as_str()) {
                                        *s = to.clone();
                                    }
                                } else {
                                    remap(it, map);
                                }
                            }
                        }
                        other => remap(other, map),
                    }
                } else {
                    remap(x, map);
                }
            }
        }
        Json::Array(a) => a.iter_mut().for_each(|x| remap(x, map)),
        _ => {}
    }
}

/// Stable reorder: elements known to `prev` in its order, new ones after, in their order.
fn order<T>(v: &mut [T], prev: &[Id], id: impl Fn(&T) -> &Id) {
    let pos: HashMap<&Id, usize> = prev.iter().enumerate().map(|(i, x)| (x, i)).collect();
    let mut keyed: Vec<(usize, usize)> = v.iter().enumerate().map(|(i, x)| (pos.get(id(x)).copied().unwrap_or(usize::MAX), i)).collect();
    keyed.sort();
    let perm: Vec<usize> = keyed.into_iter().map(|k| k.1).collect();
    // Apply the permutation.
    let mut idx: Vec<usize> = (0..v.len()).collect();
    for (target, &src) in perm.iter().enumerate() {
        let cur = idx.iter().position(|&x| x == src).unwrap();
        v.swap(target, cur);
        idx.swap(target, cur);
    }
}

fn ids<T>(v: &[T], id: impl Fn(&T) -> &Id) -> Vec<Id> {
    v.iter().map(|x| id(x).clone()).collect()
}

fn reorder(doc: &mut Document, prev: &Document) {
    order(&mut doc.spaces, &ids(&prev.spaces, |s| &s.id), |s| &s.id);
    order(&mut doc.models, &ids(&prev.models, |m| &m.id), |m| &m.id);
    fn body(m: &mut Model, p: &Model) {
        order(&mut m.bindings, &ids(&p.bindings, |b| &b.id), |b| &b.id);
        order(&mut m.processes, &ids(&p.processes, |b| &b.id), |b| &b.id);
        order(&mut m.flows, &ids(&p.flows, |b| &b.id), |b| &b.id);
        order(&mut m.events, &ids(&p.events, |b| &b.id), |b| &b.id);
        order(&mut m.equations, &ids(&p.equations, |b| &b.id), |b| &b.id);
        order(&mut m.constraints, &ids(&p.constraints, |b| &b.id), |b| &b.id);
        order(&mut m.enums, &ids(&p.enums, |b| &b.id), |b| &b.id);
        order(&mut m.functions, &ids(&p.functions, |b| &b.id), |b| &b.id);
        order(&mut m.parts, &ids(&p.parts, |b| &b.id), |b| &b.id);
        order(&mut m.objects, &ids(&p.objects, |b| &b.id), |b| &b.id);
        order(&mut m.grids, &ids(&p.grids, |b| &b.id), |b| &b.id);
        order(&mut m.frames, &ids(&p.frames, |b| &b.id), |b| &b.id);
        for o in &mut m.objects {
            if let Some(po) = p.objects.iter().find(|x| x.id == o.id) {
                body(o, po);
            }
        }
    }
    for m in &mut doc.models {
        let Some(p) = prev.models.iter().find(|x| x.id == m.id) else { continue };
        body(m, p);
    }
    for l in &mut doc.libraries {
        let Some(p) = prev.libraries.iter().find(|x| x.name == l.name) else { continue };
        order(&mut l.enums, &ids(&p.enums, |b| &b.id), |b| &b.id);
        order(&mut l.functions, &ids(&p.functions, |b| &b.id), |b| &b.id);
        order(&mut l.objects, &ids(&p.objects, |b| &b.id), |b| &b.id);
        for o in &mut l.objects {
            if let Some(po) = p.objects.iter().find(|x| x.id == o.id) {
                body(o, po);
            }
        }
    }
    order(&mut doc.presentations, &ids(&prev.presentations, |p| &p.id), |p| &p.id);
    for pr in &mut doc.presentations {
        let Some(p) = prev.presentations.iter().find(|x| x.id == pr.id) else { continue };
        order(&mut pr.observations, &ids(&p.observations, |o| &o.id), |o| &o.id);
        order(&mut pr.instruments, &ids(&p.instruments, |i| &i.id), |i| &i.id);
        order(&mut pr.views, &ids(&p.views, |v| &v.id), |v| &v.id);
        for v in &mut pr.views {
            if let Some(pv) = p.views.iter().find(|x| x.id == v.id) {
                order(&mut v.representations, &ids(&pv.representations, |r| &r.id), |r| &r.id);
            }
        }
        if let (Some(t), Some(pt)) = (&mut pr.timeline, &p.timeline) {
            order(&mut t.scenes, &ids(&pt.scenes, |s| &s.id), |s| &s.id);
        }
    }
    order(&mut doc.runs, &ids(&prev.runs, |r| &r.id), |r| &r.id);
}

/// Gives the elements of a newly lowered document the identities they had in `prev`, and
/// restores `prev`'s order. Elements `prev` does not have keep their path identities (made
/// unique if a renamed element already holds one) and are appended.
pub fn reconcile(new: Document, prev: &Document) -> Document {
    reconcile_map(new, prev).0
}

/// [`reconcile`], also answering each identity of `new` that changed and the identity it
/// took, so that data keyed by identity (a source map) can follow (HI-3.2).
pub fn reconcile_map(new: Document, prev: &Document) -> (Document, HashMap<Id, Id>) {
    let mut map = match_ids(&new, prev);
    // An unmatched element whose path identity is held by another element of `prev` (one
    // renamed by a tool) gets a fresh identity.
    let taken: HashSet<Id> = map.values().cloned().collect();
    let prev_ids = identities(prev);
    for id in identities(&new) {
        if map.contains_key(&id) {
            continue;
        }
        if taken.contains(&id) || prev_ids.contains(&id) {
            let mut k = 2;
            let fresh = loop {
                let c = format!("{id}~{k}");
                if !prev_ids.contains(&c) && !taken.contains(&c) {
                    break c;
                }
                k += 1;
            };
            map.insert(id.clone(), fresh);
        }
    }
    map.retain(|k, v| k != v);
    let mut v = serde_json::to_value(&new).expect("IR serializes");
    remap(&mut v, &map);
    let mut doc: Document = serde_json::from_value(v).expect("remapped IR deserializes");
    reorder(&mut doc, prev);
    (doc, map)
}

/// Renames an element, keeping its identity: every reference, being by identity, follows.
/// A parameter's range constraint follows its parameter's name.
pub fn rename(doc: &Document, id: &str, new_name: &str) -> Result<Document, String> {
    let mut d = doc.clone();
    let clash = |names: Vec<&String>| names.iter().any(|n| *n == new_name);
    for s in &mut d.spaces {
        if s.id == id {
            if clash(doc.spaces.iter().map(|x| &x.name).collect()) {
                return Err(format!("a space `{new_name}` exists"));
            }
            s.name = new_name.into();
            return Ok(d);
        }
    }
    for m in &mut d.models {
        if m.id == id {
            if clash(doc.models.iter().map(|x| &x.name).collect()) {
                return Err(format!("a model `{new_name}` exists"));
            }
            m.name = new_name.into();
            return Ok(d);
        }
        let taken: Vec<&String> = m.bindings.iter().map(|b| &b.name).chain(m.events.iter().map(|e| &e.name)).chain(m.functions.iter().map(|f| &f.name)).collect();
        let taken_clash = taken.iter().any(|n| *n == new_name);
        if let Some(b) = m.bindings.iter_mut().find(|b| b.id == id) {
            if taken_clash {
                return Err(format!("`{new_name}` is already declared in model `{}`", m.name));
            }
            let old = std::mem::replace(&mut b.name, new_name.into());
            for c in m.constraints.iter_mut().filter(|c| c.attached_to.as_deref() == Some(id) && c.name == format!("{old}_range")) {
                c.name = format!("{new_name}_range");
            }
            return Ok(d);
        }
        if let Some(e) = m.events.iter_mut().find(|e| e.id == id) {
            if taken_clash {
                return Err(format!("`{new_name}` is already declared in model `{}`", m.name));
            }
            e.name = new_name.into();
            return Ok(d);
        }
        if let Some(f) = m.functions.iter_mut().find(|f| f.id == id) {
            if taken_clash {
                return Err(format!("`{new_name}` is already declared in model `{}`", m.name));
            }
            f.name = new_name.into();
            return Ok(d);
        }
        if let Some(e) = m.enums.iter_mut().find(|e| e.id == id) {
            if doc.models.iter().flat_map(|x| &x.enums).any(|x| x.name == new_name) {
                return Err(format!("an enumeration `{new_name}` exists"));
            }
            e.name = new_name.into();
            return Ok(d);
        }
        if let Some(p) = m.processes.iter_mut().find(|p| p.id == id) {
            p.name = new_name.into();
            return Ok(d);
        }
        if let Some(q) = m.equations.iter_mut().find(|q| q.id == id) {
            q.name = new_name.into();
            return Ok(d);
        }
        if let Some(g) = m.grids.iter_mut().find(|g| g.id == id) {
            g.name = new_name.into();
            return Ok(d);
        }
        if let Some(f) = m.frames.iter_mut().find(|f| f.id == id) {
            f.name = new_name.into();
            return Ok(d);
        }
        if let Some(c) = m.constraints.iter_mut().find(|c| c.id == id) {
            c.name = new_name.into();
            return Ok(d);
        }
    }
    for p in &mut d.presentations {
        if p.id == id {
            p.name = new_name.into();
            return Ok(d);
        }
        if let Some(o) = p.observations.iter_mut().find(|o| o.id == id) {
            o.name = new_name.into();
            return Ok(d);
        }
        if let Some(i) = p.instruments.iter_mut().find(|i| i.id == id) {
            i.name = new_name.into();
            return Ok(d);
        }
        if let Some(v) = p.views.iter_mut().find(|v| v.id == id) {
            v.name = new_name.into();
            return Ok(d);
        }
        for v in &mut p.views {
            if let Some(r) = find_rep(&mut v.representations, id) {
                r.name = Some(new_name.into());
                return Ok(d);
            }
        }
        if let Some(t) = &mut p.timeline {
            for s in &mut t.scenes {
                if s.id == id {
                    s.name = new_name.into();
                    return Ok(d);
                }
                if let Some(b) = s.beats.iter_mut().find(|b| b.id == id) {
                    b.name = new_name.into();
                    return Ok(d);
                }
            }
        }
    }
    for r in &mut d.runs {
        if r.id == id {
            r.name = new_name.into();
            return Ok(d);
        }
    }
    Err(format!("no element `{id}`"))
}
