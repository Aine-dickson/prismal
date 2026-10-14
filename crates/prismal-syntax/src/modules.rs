//! Modules (D-099): where `use` finds a module's source text, and the shared definitions a
//! document's models see by name.

use crate::lower::ObjInfo;
use prismal_ir::{Id, Library, Use};
use std::collections::HashMap;
use std::path::PathBuf;

/// Where `use name` finds the source text of module `name` (D-099). A host supplies one: a
/// directory of files, a bundle in a browser, nothing at all.
pub trait Modules {
    /// The source text of the module, or `None` when there is no such module.
    fn source(&self, name: &str) -> Option<String>;
}

/// The first-party library (R-55, D-099): modules written in Prismal and shipped with it, in
/// the repository's `lib` directory.
pub const FIRST_PARTY: &[(&str, &str)] = &[("mechanics", include_str!("../../../lib/mechanics.prismal"))];

/// The first-party modules, and nothing else.
pub struct FirstParty;

impl Modules for FirstParty {
    fn source(&self, name: &str) -> Option<String> {
        FIRST_PARTY.iter().find(|(n, _)| *n == name).map(|(_, s)| s.to_string())
    }
}

/// No modules: every `use` is an unknown module.
pub struct NoModules;

impl Modules for NoModules {
    fn source(&self, _: &str) -> Option<String> {
        None
    }
}

/// Modules held in memory, by name.
#[derive(Clone, Debug, Default)]
pub struct MapModules(pub HashMap<String, String>);

impl Modules for MapModules {
    fn source(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

/// Modules as files under one or more directories: `a.b` is `a/b.prismal`, or `a/b.md`,
/// whose program is its code blocks (`markdown_source`). The first directory that has the
/// file wins; then the first-party modules.
#[derive(Clone, Debug, Default)]
pub struct DirModules {
    pub roots: Vec<PathBuf>,
}

impl Modules for DirModules {
    fn source(&self, name: &str) -> Option<String> {
        let rel: PathBuf = name.split('.').collect();
        for root in &self.roots {
            let plain = root.join(&rel).with_extension("prismal");
            if let Ok(s) = std::fs::read_to_string(&plain) {
                return Some(s);
            }
            let md = root.join(&rel).with_extension("md");
            if let Ok(s) = std::fs::read_to_string(&md) {
                return Some(crate::markdown_source(&s));
            }
        }
        FirstParty.source(name)
    }
}

/// The shared definitions visible by name: those of the document itself and of the modules it
/// uses, as `use` selects them.
#[derive(Clone, Debug, Default)]
pub(crate) struct Visible {
    pub(crate) objects: HashMap<String, ObjInfo>,
    pub(crate) enums: HashMap<String, (Id, Vec<String>)>,
    pub(crate) functions: HashMap<String, Id>,
}

/// Where each visible name comes from, for the message when two modules give one name.
pub(crate) type Origins = HashMap<String, String>;

impl Visible {
    /// Adds a library's definitions; `names` limits them. Returns the names already visible
    /// from another library, with that library's name, and the names asked for that the
    /// library does not define.
    pub(crate) fn add(&mut self, lib: &Library, names: Option<&[String]>, origins: &mut Origins) -> (Vec<(String, String)>, Vec<String>) {
        let wanted = |n: &str| names.is_none_or(|ns| ns.iter().any(|x| x == n));
        let mut clashes = vec![];
        let mut seen = vec![];
        let mut note = |n: &str, clashes: &mut Vec<(String, String)>| {
            seen.push(n.to_string());
            match origins.get(n) {
                Some(o) if *o != lib.name => clashes.push((n.to_string(), o.clone())),
                _ => {
                    origins.insert(n.to_string(), lib.name.clone());
                }
            }
        };
        for o in lib.objects.iter().filter(|o| wanted(&o.name)) {
            note(&o.name, &mut clashes);
            self.objects.insert(o.name.clone(), crate::lower::obj_info(o));
        }
        for e in lib.enums.iter().filter(|e| wanted(&e.name)) {
            note(&e.name, &mut clashes);
            self.enums.insert(e.name.clone(), (e.id.clone(), e.cases.clone()));
        }
        for f in lib.functions.iter().filter(|f| wanted(&f.name)) {
            note(&f.name, &mut clashes);
            self.functions.insert(f.name.clone(), f.id.clone());
        }
        let missing = names.map(|ns| ns.iter().filter(|n| !seen.contains(n)).cloned().collect()).unwrap_or_default();
        (clashes, missing)
    }

    /// The names a document's models see: its own shared definitions and its uses.
    pub(crate) fn of(uses: &[Use], libraries: &[Library]) -> Visible {
        let mut v = Visible::default();
        let mut origins = Origins::new();
        if let Some(own) = libraries.iter().find(|l| l.name == prismal_ir::SHARED) {
            v.add(own, None, &mut origins);
        }
        for u in uses {
            if let Some(l) = libraries.iter().find(|l| l.name == u.module) {
                v.add(l, u.names.as_deref(), &mut origins);
            }
        }
        v
    }
}
