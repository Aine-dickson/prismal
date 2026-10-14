//! Lowering of spaces and models to the IR (docs/spec/04-ir.md).
//!
//! Identities follow declaration paths (D-036), as the builder API does: `Model.name` for
//! bindings, `Model.event.name`, `Model.process.name`, `Model.equation.name`,
//! `Model.constraint.name`, and `Model.flow.n` for the n-th flow in source order.
//! Presentations and runs are not lowered: their IR is specified with the presentation
//! prototype (04-ir section 7).

use crate::ast::{self, Bound, ExprKind, Member, Modifier, RoleWord, TypeExpr};
use crate::{Diag, Span};
use prismal_ir::build;
use prismal_ir::*;
use std::collections::{BTreeMap, HashMap};

/// Where each IR element was written.
pub type SourceMap = BTreeMap<Id, Span>;

/// Named dimensions (MK-3.3, R-48) and base dimension symbols (MK-3.1).
pub fn named_dim(name: &str) -> Option<Dim> {
    let d = |s: &str| Dim::parse(s).expect("valid dimension");
    Some(match name {
        "1" => Dim::NONE,
        "L" => d("L"),
        "M" => d("M"),
        "T" => d("T"),
        "I" => d("I"),
        "Θ" | "Th" => d("Th"),
        "N" => d("N"),
        "J" => d("J"),
        "Length" => d("L"),
        "Mass" => d("M"),
        "Time" => d("T"),
        "Current" => d("I"),
        "Amount" => d("N"),
        "Area" => d("L^2"),
        "Volume" => d("L^3"),
        "Velocity" => d("L/T"),
        "Acceleration" => d("L/T^2"),
        "Frequency" => d("1/T"),
        "Momentum" => d("M L/T"),
        "Force" => d("M L/T^2"),
        "Energy" => d("M L^2/T^2"),
        "Power" => d("M L^2/T^3"),
        "Pressure" => d("M/L/T^2"),
        "Temperature" => d("Th"),
        // Electrical quantities (D-092).
        "Charge" => d("I T"),
        "Voltage" => d("M L^2/T^3/I"),
        "Resistance" => d("M L^2/T^3/I^2"),
        "Capacitance" => d("T^4 I^2/M/L^2"),
        "Inductance" => d("M L^2/T^2/I^2"),
        _ => return None,
    })
}

const FUNCS: [(&str, Func, usize); 16] = [
    ("sin", Func::Sin, 1),
    ("cos", Func::Cos, 1),
    ("tan", Func::Tan, 1),
    ("sqrt", Func::Sqrt, 1),
    ("exp", Func::Exp, 1),
    ("log", Func::Log, 1),
    ("abs", Func::Abs, 1),
    ("min", Func::Min, 2),
    ("max", Func::Max, 2),
    ("atan2", Func::Atan2, 2),
    ("dot", Func::Dot, 2),
    ("cross", Func::Cross, 2),
    // Of fields (D-091); `min` and `max` of one field are its smallest and largest value.
    ("laplacian", Func::Laplacian, 1),
    ("gradient", Func::Gradient, 1),
    ("integral", Func::Integral, 1),
    ("mean", Func::Mean, 1),
];

pub fn lower(file: &ast::File) -> (Document, SourceMap, Vec<Diag>) {
    lower_with(file, &crate::modules::NoModules)
}

/// Lowers a file whose `use` lines find their modules in `modules` (D-099).
pub fn lower_with(file: &ast::File, modules: &dyn crate::modules::Modules) -> (Document, SourceMap, Vec<Diag>) {
    let mut diags = vec![];
    let mut map = SourceMap::new();
    // Modules first, then the file's own shared definitions: models read both by name.
    let mut libraries: Vec<Library> = vec![];
    let (uses, visible) = uses_of(file, modules, &mut vec![], &mut libraries, &mut diags);
    let mut visible = visible;
    let shared: Vec<Member> = file.items.iter().filter_map(|it| if let ast::Item::Shared(m) = it { Some(m.clone()) } else { None }).collect();
    if !shared.is_empty() {
        let own = lower_library(SHARED, &shared, &visible, &mut diags, &mut map);
        let mut origins: crate::modules::Origins = HashMap::new();
        let mut before = crate::modules::Visible::default();
        for u in &uses {
            if let Some(l) = libraries.iter().find(|l| l.name == u.module) {
                before.add(l, u.names.as_deref(), &mut origins);
            }
        }
        let (clashes, _) = visible.add(&own, None, &mut origins);
        for (n, other) in clashes {
            let span = shared.iter().find_map(|m| shared_name(m).filter(|x| x.text == n).map(|x| x.span)).unwrap_or_default();
            diags.push(Diag::new("SX-E09", format!("`{n}` is also defined by the module `{other}`: a name means one thing; rename one, or import the module's other names only, `use {other}.{{...}}`"), span));
        }
        libraries.insert(0, own);
    }
    let mut spaces: Vec<Space> = vec![];
    for it in &file.items {
        if let ast::Item::Space(s) = it {
            if spaces.iter().any(|x| x.name == s.name.text) {
                diags.push(Diag::new("SX-E09", format!("space `{}` declared twice", s.name.text), s.name.span));
                continue;
            }
            if let Some(sp) = lower_space(s, &mut diags) {
                map.insert(sp.id.clone(), s.span);
                spaces.push(sp);
            }
        }
    }
    let mut models: Vec<Model> = vec![];
    for it in &file.items {
        if let ast::Item::Model(m) = it {
            if models.iter().any(|x| x.name == m.name.text) {
                diags.push(Diag::new("SX-E09", format!("model `{}` declared twice", m.name.text), m.name.span));
                continue;
            }
            let mut cx = ModelCx::new(m, &spaces, &mut diags, &mut map);
            cx.import(&visible);
            let model = cx.lower(m);
            models.push(model);
        }
    }
    let mut presentations = vec![];
    for it in &file.items {
        if let ast::Item::Presentation(p) = it {
            if presentations.iter().any(|x: &prismal_ir::present::Presentation| x.name == p.name.text) {
                diags.push(Diag::new("SX-E09", format!("presentation `{}` declared twice", p.name.text), p.name.span));
                continue;
            }
            if let Some(pr) = crate::lower_present::presentation(p, &models, &spaces, &visible, &mut diags, &mut map) {
                presentations.push(pr);
            }
        }
    }
    let mut runs = vec![];
    for it in &file.items {
        if let ast::Item::Run(r) = it {
            if runs.iter().any(|x: &prismal_ir::present::RunCase| x.name == r.name.text) {
                diags.push(Diag::new("SX-E09", format!("run `{}` declared twice", r.name.text), r.name.span));
                continue;
            }
            if let Some(rc) = crate::lower_present::run(r, &models, &spaces, &visible, &presentations, &mut diags, &mut map) {
                runs.push(rc);
            }
        }
    }
    let mut doc = Document::new(spaces, models);
    doc.presentations = presentations;
    doc.runs = runs;
    doc.uses = uses;
    doc.libraries = libraries;
    (doc, map, diags)
}

/// The name an object type, enumeration or function declares.
fn shared_name(m: &Member) -> Option<&ast::Name> {
    match m {
        Member::Object(o) => Some(&o.name),
        Member::Enum(e) => Some(&e.name),
        Member::Fn(f) => Some(&f.name),
        _ => None,
    }
}

/// The `use` lines of a file: each module loaded into `libraries` (with the modules it uses),
/// and the names they make visible. Two modules that give one name are reported at the second
/// `use`; so is a name asked for that the module does not define.
fn uses_of(file: &ast::File, modules: &dyn crate::modules::Modules, stack: &mut Vec<String>, libraries: &mut Vec<Library>, diags: &mut Vec<Diag>) -> (Vec<Use>, crate::modules::Visible) {
    let mut uses = vec![];
    let mut visible = crate::modules::Visible::default();
    let mut origins: crate::modules::Origins = HashMap::new();
    for it in &file.items {
        let ast::Item::Use(u) = it else { continue };
        let name = u.path.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join(".");
        if name == SHARED {
            diags.push(Diag::new("SX-E08", format!("`{SHARED}` names the definitions of the file itself; a module has another name"), u.span));
            continue;
        }
        if uses.iter().any(|x: &Use| x.module == name) {
            diags.push(Diag::new("SX-E09", format!("module `{name}` is used twice: list every name in one `use`"), u.span));
            continue;
        }
        if !load_module(&name, u.span, modules, stack, libraries, diags) {
            continue;
        }
        let names: Option<Vec<String>> = u.names.as_ref().map(|ns| ns.iter().map(|n| n.text.clone()).collect());
        let lib = libraries.iter().find(|l| l.name == name).expect("loaded");
        let (clashes, missing) = visible.add(lib, names.as_deref(), &mut origins);
        for (n, other) in clashes {
            diags.push(Diag::new("SX-E09", format!("`{n}` is defined by both `{other}` and `{name}`: a name means one thing; import it from one, `use {other}.{{{n}}}`"), u.span));
        }
        for n in missing {
            diags.push(Diag::new("SX-E03", format!("module `{name}` defines no `{n}`"), u.span));
        }
        uses.push(Use { module: name, names });
    }
    (uses, visible)
}

/// Loads module `name` and the modules it uses into `libraries`, once each. Its diagnostics
/// are reported at the `use` that names it. Returns whether it is loaded.
fn load_module(name: &str, span: Span, modules: &dyn crate::modules::Modules, stack: &mut Vec<String>, libraries: &mut Vec<Library>, diags: &mut Vec<Diag>) -> bool {
    if libraries.iter().any(|l| l.name == name) {
        return true;
    }
    if stack.iter().any(|s| s == name) {
        let cycle = stack.iter().skip_while(|s| *s != name).cloned().chain(std::iter::once(name.to_string())).collect::<Vec<_>>().join(" -> ");
        diags.push(Diag::new("SX-E08", format!("modules use each other: {cycle}; a module's definitions cannot depend on themselves"), span));
        return false;
    }
    let Some(src) = modules.source(name) else {
        diags.push(Diag::new("SX-E03", format!("no module `{name}`"), span));
        return false;
    };
    let (file, mut inner) = crate::parse(&src);
    let mut lib = None;
    if inner.is_empty() {
        stack.push(name.to_string());
        let (_, visible) = uses_of(&file, modules, stack, libraries, &mut inner);
        stack.pop();
        let shared: Vec<Member> = file.items.iter().filter_map(|it| if let ast::Item::Shared(m) = it { Some(m.clone()) } else { None }).collect();
        let mut map = SourceMap::new();
        lib = Some(lower_library(name, &shared, &visible, &mut inner, &mut map));
    }
    let ok = inner.is_empty();
    for d in inner {
        diags.push(Diag::new(d.code, format!("in module `{name}`, line {}: {}", d.span.line, d.message), span));
    }
    if ok {
        libraries.push(lib.expect("lowered"));
    }
    ok
}

/// Lowers shared definitions (D-099) as the body of a model named after the library, in the
/// space `ANY_SPACE` that linking replaces with each model's.
fn lower_library(name: &str, members: &[Member], visible: &crate::modules::Visible, diags: &mut Vec<Diag>, map: &mut SourceMap) -> Library {
    let mut any = build::space("", 3);
    any.id = ANY_SPACE.into();
    let decl = ast::ModelDecl { name: ast::Name { text: name.to_string(), span: Span::default() }, space: None, members: members.to_vec(), notes: vec![], span: Span::default() };
    let spaces: [Space; 0] = [];
    let mut cx = ModelCx::new(&decl, &spaces, diags, map);
    cx.space = Some(&any);
    cx.library = true;
    cx.import(visible);
    let m = cx.lower(&decl);
    Library { name: name.to_string(), enums: m.enums, functions: m.functions, objects: m.objects }
}

/// Lowers an expression in the scope of a model already in `doc`: names resolve to its
/// bindings and events, components to its default space. Used for run overrides and
/// observations until presentations and runs have their IR (04-ir section 7).
pub fn lower_expr(doc: &Document, model: &str, e: &ast::Expr) -> Result<Expr, Vec<Diag>> {
    let m = doc.models.iter().find(|m| m.name == model).ok_or_else(|| vec![Diag::new("SX-E03", format!("unknown model `{model}`"), e.span)])?;
    let mut diags = vec![];
    let mut map = SourceMap::new();
    let visible = crate::modules::Visible::of(&doc.uses, &doc.libraries);
    let mut cx = ModelCx::scope(m, &doc.spaces, &mut diags, &mut map);
    cx.import(&visible);
    let out = cx.expr(e, &[]);
    if diags.is_empty() {
        Ok(out)
    } else {
        Err(diags)
    }
}

fn lower_space(s: &ast::SpaceDecl, diags: &mut Vec<Diag>) -> Option<Space> {
    if s.kind.text != "euclidean" {
        diags.push(Diag::new("SX-E06", format!("space kind `{}`: v0 has only `euclidean(n)`", s.kind.text), s.kind.span));
        return None;
    }
    let n = match s.args.as_slice() {
        [ast::Arg { name: None, value: ast::Expr { kind: ExprKind::Num(n, None), .. }, every: None, ensemble: None, on: None }] if (1.0..=3.0).contains(n) && n.fract() == 0.0 => *n as u32,
        _ => {
            diags.push(Diag::new("SX-E02", "a space is declared `euclidean(n)` with n = 1, 2 or 3", s.span));
            return None;
        }
    };
    let mut sp = build::space(&s.name.text, n);
    sp.notes = s.notes.clone();
    Some(sp)
}

/// An object type's names, for member expressions (D-055).
#[derive(Clone, Debug, Default)]
pub(crate) struct ObjInfo {
    pub(crate) id: Id,
    pub(crate) bindings: HashMap<String, Id>,
    pub(crate) parts: HashMap<String, PartInfo>,
    /// A relation type's endpoints: identity and the object type of its collection (D-058).
    pub(crate) ends: HashMap<String, (Id, String)>,
}

/// A part: its identity, its object type's name, and whether it is a collection.
#[derive(Clone, Debug)]
pub(crate) struct PartInfo {
    pub(crate) id: Id,
    pub(crate) object: String,
    pub(crate) many: bool,
}

pub(crate) struct ModelCx<'a> {
    pub(crate) name: String,
    /// Object types of the model by name, and the parts of the scope (D-055).
    pub(crate) objects: HashMap<String, ObjInfo>,
    pub(crate) parts: HashMap<String, PartInfo>,
    /// Loop variables in scope: the member's name and its object type's name.
    pub(crate) vars: Vec<(String, String)>,
    /// In the overrides of a collection, `index` is the member's number.
    index_ok: bool,
    /// Lowering an object type's body, where object types are not declared.
    in_object: bool,
    /// In a relation type's body, its endpoints (D-058).
    ends: HashMap<String, (Id, String)>,
    space: Option<&'a Space>,
    spaces: &'a [Space],
    pub(crate) bindings: HashMap<String, Id>,
    pub(crate) events: HashMap<String, Id>,
    /// Declared enumerations by name: identity and cases (D-049).
    pub(crate) enums: HashMap<String, (Id, Vec<String>)>,
    /// Declared functions by name (D-048).
    pub(crate) functions: HashMap<String, Id>,
    /// Grids by name (D-091).
    pub(crate) grids: HashMap<String, Id>,
    /// While lowering an event with payloads: each value payload's name, the event, and its
    /// component when the event carries several (D-050, D-059). Member payloads are in `vars`.
    payload: Vec<(String, Id, Option<usize>)>,
    pub(crate) diags: &'a mut Vec<Diag>,
    pub(crate) map: &'a mut SourceMap,
    flows: usize,
    model: Model,
    /// Names of shared definitions made visible by `import` (D-099): a declaration of the
    /// model with one of these names is its own, not a second declaration.
    imported: std::collections::HashSet<String>,
    /// The shared definitions imported, passed on to object types' bodies (D-099).
    shared: crate::modules::Visible,
    /// Lowering a library's shared definitions: its object types also read its functions and
    /// enumerations (D-099).
    library: bool,
}

impl<'a> ModelCx<'a> {
    /// Makes shared definitions visible by name (D-099); the model's own definitions of the
    /// same names win.
    pub(crate) fn import(&mut self, v: &crate::modules::Visible) {
        self.shared.objects.extend(v.objects.iter().map(|(k, x)| (k.clone(), x.clone())));
        self.shared.enums.extend(v.enums.iter().map(|(k, x)| (k.clone(), x.clone())));
        self.shared.functions.extend(v.functions.iter().map(|(k, x)| (k.clone(), x.clone())));
        for (n, o) in &v.objects {
            if !self.objects.contains_key(n) {
                self.objects.insert(n.clone(), o.clone());
                self.imported.insert(n.clone());
            }
        }
        for (n, e) in &v.enums {
            if !self.enums.contains_key(n) {
                self.enums.insert(n.clone(), e.clone());
                self.imported.insert(n.clone());
            }
        }
        for (n, f) in &v.functions {
            if !self.functions.contains_key(n) {
                self.functions.insert(n.clone(), f.clone());
                self.imported.insert(n.clone());
            }
        }
    }

    /// The scope of a lowered model, for expressions written outside it (presentations, runs).
    pub(crate) fn scope(m: &Model, spaces: &'a [Space], diags: &'a mut Vec<Diag>, map: &'a mut SourceMap) -> ModelCx<'a> {
        let mut objects: HashMap<String, ObjInfo> = m.objects.iter().map(|o| (o.name.clone(), obj_info(o))).collect();
        // A relation type's endpoints, with the object types of their collections (D-058).
        for o in &m.objects {
            for e in &o.ends {
                // The last part of a part path (D-065) is declared in the model or an object type.
                let last = e.over.rsplit(prismal_ir::PART_PATH).next().unwrap_or(&e.over);
                let ty = std::iter::once(m).chain(&m.objects).flat_map(|x| &x.parts).find(|p| p.id == last).map(|p| p.object.rsplit('.').next().unwrap_or(&p.object).to_string()).unwrap_or_default();
                objects.get_mut(&o.name).unwrap().ends.insert(e.name.clone(), (e.id.clone(), ty));
            }
        }
        let parts = obj_info(m).parts;
        ModelCx {
            objects,
            parts,
            vars: vec![],
            index_ok: false,
            in_object: false,
            ends: HashMap::new(),
            name: m.name.clone(),
            space: m.default_space.as_ref().and_then(|s| spaces.iter().find(|x| &x.id == s)),
            spaces,
            bindings: m.bindings.iter().map(|b| (b.name.clone(), b.id.clone())).collect(),
            events: m.events.iter().map(|ev| (ev.name.clone(), ev.id.clone())).collect(),
            enums: m.enums.iter().map(|e| (e.name.clone(), (e.id.clone(), e.cases.clone()))).collect(),
            functions: m.functions.iter().map(|f| (f.name.clone(), f.id.clone())).collect(),
            grids: m.grids.iter().map(|g| (g.name.clone(), g.id.clone())).collect(),
            payload: vec![],
            diags,
            map,
            flows: 0,
            model: build::ModelBuilder::new(&m.name).finish(),
            imported: Default::default(),
            shared: Default::default(),
            library: false,
        }
    }

    /// The dimension of a space of the document (D-100).
    pub(crate) fn space_dimension(&self, id: &str) -> Option<u32> {
        self.spaces.iter().find(|s| s.id == id).map(|s| s.dimension)
    }

    pub(crate) fn space_by_name(&self, name: &str) -> Option<Id> {
        self.spaces.iter().find(|s| s.name == name).map(|s| s.id.clone())
    }

    fn new(m: &ast::ModelDecl, spaces: &'a [Space], diags: &'a mut Vec<Diag>, map: &'a mut SourceMap) -> ModelCx<'a> {
        let name = m.name.text.clone();
        let space = match &m.space {
            Some(s) => match spaces.iter().find(|x| x.name == s.text) {
                Some(sp) => Some(sp),
                None => {
                    diags.push(Diag::new("SX-E03", format!("unknown space `{}`", s.text), s.span));
                    None
                }
            },
            None => None,
        };
        let model = build::ModelBuilder::new(&name).finish();
        ModelCx {
            name,
            objects: HashMap::new(),
            parts: HashMap::new(),
            vars: vec![],
            index_ok: false,
            in_object: false,
            ends: HashMap::new(),
            space,
            spaces,
            bindings: HashMap::new(),
            events: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            grids: HashMap::new(),
            payload: vec![],
            diags,
            map,
            flows: 0,
            model,
            imported: Default::default(),
            shared: Default::default(),
            library: false,
        }
    }

    /// The names of an object type or model as written: its bindings and parts (D-055).
    fn declare_names(&mut self, id: &str, members: &[Member]) -> ObjInfo {
        let mut info = ObjInfo { id: id.to_string(), ..Default::default() };
        for mem in members {
            match mem {
                Member::Decl(d) => {
                    info.bindings.insert(d.name.text.clone(), format!("{id}.{}", d.name.text));
                }
                Member::Parts(ps) => {
                    for p in ps {
                        let pi = PartInfo { id: format!("{id}.part.{}", p.name.text), object: p.object.text.clone(), many: p.count.is_some() || p.capacity.is_some() || p.unbounded };
                        if info.parts.insert(p.name.text.clone(), pi).is_some() {
                            self.err("SX-E09", format!("part `{}` declared twice", p.name.text), p.name.span);
                        }
                        if info.bindings.contains_key(&p.name.text) {
                            self.err("SX-E09", format!("`{}` is both a binding and a part", p.name.text), p.name.span);
                        }
                    }
                }
                _ => {}
            }
        }
        info
    }

    /// Lowers an object type declared in this model (D-055).
    fn object(&mut self, o: &ast::ObjectDecl) {
        if self.in_object {
            self.err("SX-E06", "object types are declared in the model, not inside another object type", o.name.span);
            return;
        }
        let info = self.objects[&o.name.text].clone();
        let decl = ast::ModelDecl { name: ast::Name { text: info.id.clone(), span: o.name.span }, space: None, members: o.members.clone(), notes: o.notes.clone(), span: o.span };
        let objects = self.objects.clone();
        let shared = self.shared.clone();
        let own = self.library.then(|| (self.functions.clone(), self.enums.clone()));
        let mut sub = ModelCx::new(&decl, self.spaces, &mut *self.diags, &mut *self.map);
        sub.space = self.space;
        // Shared definitions are visible in object types too (D-099); in a library, so are
        // the library's own functions and enumerations.
        sub.import(&shared);
        if let Some((functions, enums)) = own {
            sub.import(&crate::modules::Visible { objects: HashMap::new(), enums, functions });
        }
        sub.objects = objects;
        sub.parts = info.parts.clone();
        sub.in_object = true;
        sub.ends = info.ends.clone();
        let mut ty = sub.lower(&decl);
        ty.name = o.name.text.clone();
        ty.undirected = o.undirected;
        // A relation type's endpoints are members of the model's collections (D-058).
        for (n, path) in &o.ends {
            let over = self.end_path(path, false).map(|(id, _)| id).unwrap_or_default();
            ty.ends.push(prismal_ir::End { id: format!("{}.{}", info.id, n.text), name: n.text.clone(), over });
        }
        self.model.objects.push(ty);
    }

    /// The collection of an endpoint, `balls` or `left.atoms` (D-058, D-065): its part path
    /// and object type.
    fn end_path(&mut self, path: &[ast::Name], report: bool) -> Option<(Id, String)> {
        let text = path.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join(".");
        let span = Span { end: path[path.len() - 1].span.end, ..path[0].span };
        match self.part_ref(&text) {
            Ok(p) => Some((p.id, p.object)),
            Err(m) => {
                if report {
                    self.err(if m.starts_with("unknown") { "SX-E03" } else { "SX-E08" }, m, span);
                }
                None
            }
        }
    }

    /// A part named by a path through contained objects, `left.atoms` (D-065), or by its
    /// name: its part path (identities joined by `/`), object type and whether it is a
    /// collection.
    pub(crate) fn part_ref(&self, text: &str) -> Result<PartInfo, String> {
        let names: Vec<&str> = text.split('.').collect();
        let mut parts = &self.parts;
        let mut ids = vec![];
        for (k, n) in names.iter().enumerate() {
            let Some(p) = parts.get(*n) else { return Err(format!("unknown collection `{n}`")) };
            ids.push(p.id.clone());
            if k + 1 == names.len() {
                return Ok(PartInfo { id: ids.join(&prismal_ir::PART_PATH.to_string()), object: p.object.clone(), many: p.many });
            }
            if p.many {
                return Err(format!("`{n}` is a collection: a path goes through contained objects only (`{n}[1]` is one member)"));
            }
            parts = match self.objects.get(&p.object) {
                Some(o) => &o.parts,
                None => return Err(format!("unknown collection `{text}`")),
            };
        }
        Err(format!("unknown collection `{text}`"))
    }

    /// The dotted text of a path of names, `left.atoms`, or `None`.
    fn dotted(e: &ast::Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Name(n) => Some(n.clone()),
            ExprKind::Field(x, n) => Some(format!("{}.{}", Self::dotted(x)?, n.text)),
            _ => None,
        }
    }

    /// Lowers the parts of the scope (D-055).
    fn parts_decl(&mut self, ps: &[ast::PartDecl]) {
        for p in ps {
            let info = self.parts[&p.name.text].clone();
            self.map.insert(info.id.clone(), p.span);
            let Some(obj) = self.objects.get(&p.object.text).cloned() else {
                // Kept, so that its uses are known and not reported again.
                self.err("SX-E03", format!("unknown object type `{}`", p.object.text), p.object.span);
                let object = format!("{}.{}", self.name, p.object.text);
                self.model.parts.push(Part { id: info.id, name: p.name.text.clone(), object, count: p.count, capacity: p.capacity, unbounded: p.unbounded, overrides: vec![], notes: p.notes.clone() });
                continue;
            };
            if p.count == Some(0) && p.capacity.is_none() && !p.unbounded {
                self.err("SX-E08", "a collection has at least one member", p.span);
            }
            if p.capacity == Some(0) {
                self.err("SX-E08", "a collection holds at least one member", p.span);
            }
            let mut overrides = vec![];
            for (n, e) in &p.overrides {
                // An endpoint of a starting relation is given a member (D-058).
                if let Some((end, _)) = obj.ends.get(&n.text).cloned() {
                    self.index_ok = info.many;
                    let value = match self.member_expr(e, &[]) {
                        Some((m, _)) => m,
                        None => {
                            self.err("SX-E08", format!("the endpoint `{}` is a member: `{} = balls[1]`", n.text, n.text), e.span);
                            Self::placeholder()
                        }
                    };
                    self.index_ok = false;
                    overrides.push(prismal_ir::present::Override { binding: end, value });
                    continue;
                }
                let Some(b) = obj.bindings.get(&n.text).cloned() else {
                    self.err("SX-E03", format!("an object `{}` has no binding `{}`", p.object.text, n.text), n.span);
                    continue;
                };
                self.index_ok = info.many;
                let value = self.expr(e, &[]);
                self.index_ok = false;
                overrides.push(prismal_ir::present::Override { binding: b, value });
            }
            self.model.parts.push(Part { id: info.id, name: p.name.text.clone(), object: obj.id, count: p.count, capacity: p.capacity, unbounded: p.unbounded, overrides, notes: p.notes.clone() });
        }
    }

    /// The object type of the member an expression selects, without reporting anything.
    fn quiet_type(&self, e: &ast::Expr, locals: &[String]) -> Option<String> {
        match &e.kind {
            ExprKind::Name(n) if locals.contains(n) => None,
            ExprKind::Name(n) => {
                if let Some((_, t)) = self.vars.iter().rev().find(|(v, _)| v == n) {
                    return Some(t.clone());
                }
                if !self.bindings.contains_key(n) {
                    if let Some((_, t)) = self.ends.get(n) {
                        return Some(t.clone());
                    }
                }
                self.parts.get(n).filter(|p| !p.many).map(|p| p.object.clone())
            }
            ExprKind::Index(x, _) => match Self::dotted(x) {
                Some(n) if !locals.contains(&n) => self.part_ref(&n).ok().filter(|p| p.many).map(|p| p.object),
                _ => None,
            },
            ExprKind::Field(x, n) => {
                let t = self.quiet_type(x, locals)?;
                self.objects.get(&t)?.ends.get(&n.text).map(|(_, t)| t.clone())
            }
            ExprKind::Call(f, args) if args.len() == 1 => match &f.kind {
                ExprKind::Field(x, n) if n.text == "other" => {
                    let t = self.quiet_type(x, locals)?;
                    self.objects.get(&t)?.ends.values().next().map(|(_, t)| t.clone())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// `s.other(o)` and `s.has(o)` on a relation `s` (D-064): the method, the relation and
    /// the member, without reporting anything.
    fn rel_method<'e>(&self, e: &'e ast::Expr, locals: &[String]) -> Option<(&'e str, &'e ast::Expr, &'e ast::Expr)> {
        let ExprKind::Call(f, args) = &e.kind else { return None };
        let ExprKind::Field(x, n) = &f.kind else { return None };
        let [arg] = args.as_slice() else { return None };
        if !matches!(n.text.as_str(), "other" | "has") || arg.name.is_some() {
            return None;
        }
        let t = self.quiet_type(x, locals)?;
        let o = self.objects.get(&t)?;
        if o.ends.is_empty() || o.bindings.contains_key(&n.text) {
            return None;
        }
        Some((n.text.as_str(), x, &arg.value))
    }

    /// The member an expression selects (`b`, `ball`, `row[2]`) and its object type (D-055).
    fn member_expr(&mut self, e: &ast::Expr, locals: &[String]) -> Option<(Expr, String)> {
        if let Some(("other", rel, of)) = self.rel_method(e, locals) {
            let (rel, rty) = self.member_expr(rel, locals)?;
            let (of, _) = self.member_expr(of, locals)?;
            let ety = self.objects.get(&rty)?.ends.values().next()?.1.clone();
            return Some((Expr::Other { other: Box::new(of), rel: Box::new(rel) }, ety));
        }
        match &e.kind {
            // An endpoint inside its relation type, and of a relation: `a`, `s.a` (D-058).
            ExprKind::Name(n) if !locals.contains(n) && !self.bindings.contains_key(n) && self.ends.contains_key(n) => {
                let (end, ty) = self.ends[n].clone();
                Some((Expr::End { end, of: None }, ty))
            }
            ExprKind::Field(..) if Self::dotted(e).is_some_and(|d| d.contains('.') && self.part_ref(&d).is_ok_and(|p| !p.many) && !locals.iter().any(|l| d.starts_with(&format!("{l}.")))) => {
                let p = self.part_ref(&Self::dotted(e).unwrap()).ok()?;
                Some((Expr::Part { part: p.id }, p.object))
            }
            ExprKind::Field(x, n) if self.quiet_type(x, locals).is_some_and(|t| self.objects.get(&t).is_some_and(|o| o.ends.contains_key(&n.text))) => {
                let (of, ty) = self.member_expr(x, locals)?;
                let (end, ety) = self.objects.get(&ty)?.ends.get(&n.text)?.clone();
                Some((Expr::End { end, of: Some(Box::new(of)) }, ety))
            }
            ExprKind::Name(n) if !locals.contains(n) => {
                if let Some((_, ty)) = self.vars.iter().rev().find(|(v, _)| v == n) {
                    return Some((Expr::Var { var: n.clone() }, ty.clone()));
                }
                let p = self.parts.get(n)?.clone();
                if p.many {
                    self.err("SX-E08", format!("`{n}` is a collection: select a member, `{n}[1]`"), e.span);
                }
                Some((Expr::Part { part: p.id }, p.object))
            }
            ExprKind::Index(x, i) => {
                let n = Self::dotted(x)?;
                let p = self.part_ref(&n).ok()?;
                if !p.many {
                    self.err("SX-E08", format!("`{n}` is one object, not a collection"), e.span);
                }
                let index = self.expr(i, locals);
                Some((Expr::Item { item: p.id, index: Box::new(index) }, p.object))
            }
            _ => None,
        }
    }

    /// `sum(e for b in row [if c])` and the other aggregates; `count(row)` (D-055).
    fn aggregate(&mut self, name: &str, args: &[ast::Arg], span: Span, locals: &[String]) -> Option<Expr> {
        let agg = match name {
            "sum" => Agg::Sum,
            "min" => Agg::Min,
            "max" => Agg::Max,
            "any" => Agg::Any,
            "all" => Agg::All,
            "count" => Agg::Count,
            _ => return None,
        };
        let [arg] = args else { return None };
        match &arg.value.kind {
            ExprKind::Aggregate { body, var, over, filter, .. } => {
                let Some(p) = self.part_ref(&over.text).ok() else {
                    self.err("SX-E03", format!("unknown collection `{}`", over.text), over.span);
                    return Some(Self::placeholder());
                };
                if !p.many {
                    self.err("SX-E08", format!("`{}` is one object; an aggregate goes over a collection", over.text), over.span);
                }
                self.vars.push((var.text.clone(), p.object.clone()));
                let body = if agg == Agg::Count { None } else { Some(Box::new(self.expr(body, locals))) };
                let filter = filter.as_ref().map(|f| Box::new(self.expr(f, locals)));
                self.vars.pop();
                Some(Expr::Aggregate { aggregate: agg, var: var.text.clone(), over: p.id, body, filter })
            }
            // `count(drops)`, `count(left.atoms)` (D-065).
            _ if agg == Agg::Count && Self::dotted(&args[0].value).is_some_and(|n| self.part_ref(&n).is_ok_and(|p| p.many)) => {
                let p = self.part_ref(&Self::dotted(&args[0].value).unwrap()).ok()?;
                Some(Expr::Aggregate { aggregate: agg, var: "_".into(), over: p.id, body: None, filter: None })
            }
            _ if agg == Agg::Count => {
                self.err("SX-E08", "`count` takes a collection: `count(row)`, or `count(b for b in row if c)`", span);
                Some(Self::placeholder())
            }
            _ => None,
        }
    }

    pub(crate) fn err(&mut self, code: &'static str, msg: impl Into<String>, span: Span) {
        self.diags.push(Diag::new(code, msg, span));
    }

    fn lower(&mut self, m: &ast::ModelDecl) -> Model {
        self.model.default_space = self.space.map(|s| s.id.clone());
        self.model.notes = m.notes.clone();
        self.map.insert(self.model.id.clone(), m.span);
        // Object types and parts first: member expressions anywhere may name them (D-055).
        let own = self.declare_names(&self.name.clone(), &m.members);
        self.parts = own.parts;
        for mem in &m.members {
            if let Member::Object(o) = mem {
                let id = format!("{}.{}", self.name, o.name.text);
                let info = self.declare_names(&id, &o.members);
                if self.objects.insert(o.name.text.clone(), info).is_some() && !self.imported.remove(&o.name.text) {
                    self.err("SX-E09", format!("object type `{}` declared twice", o.name.text), o.name.span);
                }
            }
        }
        // Endpoints once every object type is known: a collection may be a part of a
        // contained object, `a in left.atoms` (D-065).
        for mem in &m.members {
            if let Member::Object(o) = mem {
                let id = format!("{}.{}", self.name, o.name.text);
                for (n, path) in &o.ends {
                    // An unknown collection is reported here; its endpoint is still known.
                    let object = self.end_path(path, true).map(|(_, t)| t).unwrap_or_default();
                    let info = self.objects.get_mut(&o.name.text).expect("declared");
                    if info.bindings.contains_key(&n.text) || info.ends.insert(n.text.clone(), (format!("{id}.{}", n.text), object)).is_some() {
                        self.err("SX-E09", format!("`{}` is declared twice in `{}`", n.text, o.name.text), n.span);
                    }
                }
            }
        }
        // Declared names first, so that declarations may refer to later ones.
        let mut event_names: HashMap<String, Span> = HashMap::new();
        for mem in &m.members {
            match mem {
                Member::Decl(d) => {
                    self.bindings.insert(d.name.text.clone(), format!("{}.{}", self.name, d.name.text));
                }
                Member::Event(e) => self.declare_event(e, &mut event_names),
                Member::Process(p) => {
                    for e in &p.events {
                        self.declare_event(e, &mut event_names);
                    }
                }
                Member::Enum(e) => {
                    let id = format!("{}.enum.{}", self.name, e.name.text);
                    let cases = e.cases.iter().map(|c| c.text.clone()).collect();
                    if self.enums.insert(e.name.text.clone(), (id, cases)).is_some() && !self.imported.remove(&e.name.text) {
                        self.err("SX-E09", format!("enumeration `{}` declared twice", e.name.text), e.name.span);
                    }
                }
                Member::Fn(f) => {
                    let id = format!("{}.fn.{}", self.name, f.name.text);
                    if self.functions.insert(f.name.text.clone(), id).is_some() && !self.imported.remove(&f.name.text) {
                        self.err("SX-E09", format!("function `{}` declared twice", f.name.text), f.name.span);
                    }
                }
                Member::Grid { name, .. } => {
                    if self.grids.insert(name.text.clone(), format!("{}.grid.{}", self.name, name.text)).is_some() {
                        self.err("SX-E09", format!("grid `{}` declared twice", name.text), name.span);
                    }
                }
                _ => {}
            }
        }
        // One namespace for bindings, functions and cases: a name means one thing.
        for mem in &m.members {
            match mem {
                Member::Fn(f) if self.bindings.contains_key(&f.name.text) => {
                    self.err("SX-E09", format!("`{}` is both a binding and a function", f.name.text), f.name.span)
                }
                Member::Enum(e) => {
                    for c in &e.cases {
                        if self.bindings.contains_key(&c.text) || self.functions.contains_key(&c.text) {
                            self.err("SX-E09", format!("case `{}` has the name of a binding or function", c.text), c.span);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut processes: HashMap<String, Span> = HashMap::new();
        let mut names: HashMap<String, Span> = HashMap::new();
        for mem in &m.members {
            match mem {
                Member::Decl(d) => self.decl(d),
                Member::Fn(f) => self.function(f),
                Member::Enum(e) => {
                    let (id, cases) = self.enums[&e.name.text].clone();
                    self.map.insert(id.clone(), e.span);
                    if !self.model.enums.iter().any(|x| x.id == id) {
                        self.model.enums.push(EnumDecl { id, name: e.name.text.clone(), cases, notes: e.notes.clone() });
                    }
                }
                Member::Flow(f) => self.flow(f, None),
                Member::Grid { name, args, span } => self.grid(name, args, *span),
                // After the declarations: below.
                Member::Boundary { .. } => {}
                Member::Process(p) => {
                    if processes.insert(p.name.text.clone(), p.span).is_some() {
                        self.err("SX-E09", format!("process `{}` declared twice", p.name.text), p.name.span);
                    }
                    // MK-14.1: the IR carries the process kind; the working syntax has no
                    // spelling for it, so a process with flows is continuous, otherwise discrete.
                    let kind = if p.flows.is_empty() { ProcessKind::Discrete } else { ProcessKind::Continuous };
                    let id = format!("{}.process.{}", self.name, p.name.text);
                    self.map.insert(id.clone(), p.span);
                    self.model.processes.push(Process { id: id.clone(), name: p.name.text.clone(), kind, notes: p.notes.clone() });
                    for f in &p.flows {
                        self.flow(f, Some(&id));
                    }
                    for e in &p.events {
                        self.event(e, Some(&id));
                    }
                }
                Member::Event(e) => self.event(e, None),
                Member::Equation(eq) => {
                    if names.insert(format!("equation {}", eq.name.text), eq.span).is_some() {
                        self.err("SX-E09", format!("equation `{}` declared twice", eq.name.text), eq.name.span);
                    }
                    let lhs = self.expr(&eq.lhs, &[]);
                    let rhs = self.expr(&eq.rhs, &[]);
                    let tol = eq.checked.as_ref().map(|t| self.expr(t, &[]));
                    let role = if tol.is_some() { EquationRole::Check } else { EquationRole::Display };
                    let id = format!("{}.equation.{}", self.name, eq.name.text);
                    self.map.insert(id.clone(), eq.span);
                    self.model.equations.push(Equation { id, name: eq.name.text.clone(), lhs, rhs, role, tol });
                }
                Member::Constraint(c) => {
                    let name = match &c.name {
                        Some(n) => n.text.clone(),
                        None => format!("c{}", self.model.constraints.len() + 1),
                    };
                    let cond = self.expr(&c.cond, &[]);
                    let tol = c.within.as_ref().map(|t| self.expr(t, &[]));
                    // MK-12.4: constraints other than parameter ranges default to `report`.
                    let policy = match c.policy.as_ref().map(|p| p.text.as_str()) {
                        None | Some("report") => Policy::Report,
                        Some("reject") => Policy::Reject,
                        Some("stop") => Policy::Stop,
                        Some("enforce") => Policy::Enforce,
                        Some(other) => {
                            let span = c.policy.as_ref().unwrap().span;
                            self.err("SX-E02", format!("unknown policy `{other}`: expected `reject`, `report`, `stop` or `enforce`"), span);
                            Policy::Report
                        }
                    };
                    self.push_constraint(&name, cond, tol, policy, None, c.span);
                    if let Some(f) = &c.force {
                        let force = match self.expr(&ast::Expr { kind: ast::ExprKind::Name(f.text.clone()), span: f.span }, &[]) {
                            Expr::Ref { r#ref } => Some(r#ref),
                            _ => {
                                self.err("SX-E03", format!("`{}` is not a binding of the model: `with` names the unknown that holds the constraint", f.text), f.span);
                                None
                            }
                        };
                        self.model.constraints.last_mut().expect("pushed").force = force;
                    }
                }
                Member::Object(o) => self.object(o),
                Member::Parts(ps) => self.parts_decl(ps),
            }
        }
        // Boundary conditions attach to fields declared anywhere in the model (D-091).
        for mem in &m.members {
            if let Member::Boundary { field, periodic, fixed, .. } = mem {
                self.boundary(field, *periodic, fixed);
            }
        }
        std::mem::replace(&mut self.model, build::ModelBuilder::new("").finish())
    }

    /// A declared function (MK-10.3, D-048): its parameters are lambda parameters of its body.
    fn function(&mut self, f: &ast::FnDecl) {
        let id = format!("{}.fn.{}", self.name, f.name.text);
        let locals: Vec<String> = f.params.iter().map(|(n, _)| n.text.clone()).collect();
        let params = f.params.iter().map(|(n, t)| FnParam { name: n.text.clone(), ty: self.ty(t) }).collect();
        let result = self.ty(&f.result);
        let body = self.expr(&f.body, &locals);
        self.map.insert(id.clone(), f.span);
        if !self.model.functions.iter().any(|x| x.id == id) {
            self.model.functions.push(FunctionDecl { id, name: f.name.text.clone(), params, result, body, notes: f.notes.clone() });
        }
    }

    fn declare_event(&mut self, e: &ast::EventDecl, seen: &mut HashMap<String, Span>) {
        if seen.insert(e.name.text.clone(), e.span).is_some() {
            self.err("SX-E09", format!("event `{}` declared twice", e.name.text), e.name.span);
        }
        self.events.insert(e.name.text.clone(), format!("{}.event.{}", self.name, e.name.text));
    }

    fn push_constraint(&mut self, name: &str, cond: Expr, tol: Option<Expr>, policy: Policy, attached_to: Option<Id>, span: Span) {
        let id = format!("{}.constraint.{}", self.name, name);
        if self.model.constraints.iter().any(|c| c.id == id) {
            self.err("SX-E09", format!("constraint `{name}` declared twice"), span);
        }
        self.map.insert(id.clone(), span);
        self.model.constraints.push(Constraint { id, name: name.into(), cond, tol, policy, attached_to, force: None });
    }

    // ------------------------------------------------------------ declarations

    fn decl(&mut self, d: &ast::Decl) {
        let id = format!("{}.{}", self.name, d.name.text);
        let role = match d.role {
            RoleWord::Const => Role::Constant,
            RoleWord::Param => Role::Parameter,
            RoleWord::Input => Role::Input,
            RoleWord::State => Role::Continuous,
            RoleWord::Discrete => Role::Discrete,
            RoleWord::Derived => Role::Derived,
            RoleWord::Unknown => Role::Unknown,
        };
        let mut ty = self.ty(&d.ty);
        let (mut init, mut def) = (None, None);
        let mut locals: Vec<String> = d.params.iter().flatten().map(|(n, _)| n.text.clone()).collect();
        locals.extend(d.over.iter().map(|(x, _)| x.text.clone()));
        let value = d.value.as_ref().map(|v| self.expr(v, &locals));
        if let Some((x, g)) = &d.over {
            // A field over a grid (D-091): its value a function of the cell's coordinate.
            let grid = match self.grids.get(&g.text) {
                Some(id) => id.clone(),
                None => {
                    self.err("SX-E03", format!("unknown grid `{}`", g.text), g.span);
                    format!("{}.grid.{}", self.name, g.text)
                }
            };
            if !matches!(role, Role::Continuous | Role::Derived) {
                self.err("SX-E08", "a field is `state` or `derived` (MK-9.4)", d.name.span);
            }
            ty = Type::Field { grid, of: Box::new(ty) };
            let body = value.map(|body| build::lambda_named(&[x.text.as_str()], vec![Type::real()], body));
            if role == Role::Derived {
                def = body;
            } else {
                init = body;
            }
        } else if let Some(ps) = &d.params {
            if role != Role::Derived {
                self.err("SX-E08", "only derived bindings may have parameters (MK-10.7); use `derived`", d.name.span);
            }
            let params: Vec<Type> = ps.iter().map(|(_, t)| self.ty(t)).collect();
            ty = Type::func(params.clone(), ty);
            let names: Vec<&str> = locals.iter().map(|s| s.as_str()).collect();
            def = value.map(|body| build::lambda_named(&names, params, body));
        } else if role == Role::Derived {
            def = value;
        } else if role == Role::Input {
            // D-051: an input's value is its default until the environment supplies one.
            init = value;
        } else {
            init = value;
        }
        let mut b = Binding {
            id: id.clone(),
            name: d.name.text.clone(),
            role,
            ty,
            init,
            def,
            intervenable: None,
            private: false,
            display: Display::default(),
            notes: d.notes.clone(),
            when: None,
            combine: None,
            boundary: None,
        };
        for m in &d.modifiers {
            match m {
                Modifier::Intervenable => b.intervenable = Some(true),
                Modifier::Private => b.private = true,
                Modifier::Combine(c) => {
                    if !["sum", "product", "min", "max", "any", "all"].contains(&c.text.as_str()) {
                        self.err("SX-E02", format!("unknown combination `{}`: `sum`, `product`, `min`, `max`, `any` or `all`", c.text), c.span);
                    } else if role != Role::Discrete {
                        self.err("SX-E06", "a combination is declared on discrete state, which `contribute` changes at events (MK-14.9)", c.span);
                    } else {
                        b.combine = Some(c.text.clone());
                    }
                }
                Modifier::Symbol(s) => b.display.symbol = Some(s.clone()),
                Modifier::Unit(u) => match Unit::parse(&u.text) {
                    Ok(_) => b.display.unit = Some(u.text.clone()),
                    Err(e) => self.err("SX-E05", e, u.span),
                },
            }
        }
        self.map.insert(id.clone(), d.span);
        self.model.bindings.push(b);
        // `where cond` and `in I`: a `reject` constraint attached to the binding (MK-12.4).
        if let Some(range) = &d.range {
            let (cond, span) = match range {
                ast::Range::Where(e) => (self.expr(e, &[]), e.span),
                ast::Range::In(i) => (self.interval(build::r(&id), i, &[]), i.span),
            };
            if role != Role::Parameter {
                self.err("SX-E08", "a range (`in` or `where`) is written on a parameter (MK-12.4); use `constraint`", span);
            }
            self.push_constraint(&format!("{}_range", d.name.text), cond, None, Policy::Reject, Some(id), span);
        }
    }

    /// `grid rod(x: [0 m, 1 m], cells: 100)` (MK-9.5, D-091).
    fn grid(&mut self, name: &ast::Name, args: &[ast::Arg], span: Span) {
        let id = format!("{}.grid.{}", self.name, name.text);
        self.map.insert(id.clone(), span);
        let (mut xr, mut yr, mut cells) = (None, None, vec![]);
        for a in args {
            match a.name.as_ref().map(|n| n.text.as_str()) {
                Some(w @ ("x" | "y")) => match &a.value.kind {
                    ExprKind::List(items) if items.len() == 2 => {
                        let r = Some((self.expr(&items[0], &[]), self.expr(&items[1], &[])));
                        if w == "x" {
                            xr = r;
                        } else {
                            yr = r;
                        }
                    }
                    _ => self.err("SX-E02", format!("a grid's axis is an interval: `{w}: [0 m, 1 m]`"), a.value.span),
                },
                // `cells: 100`, or one count per axis in the plane, `cells: (50, 25)`.
                Some("cells") => match &a.value.kind {
                    ExprKind::Tuple(items) => cells = items.iter().map(|e| self.expr(e, &[])).collect(),
                    _ => cells = vec![self.expr(&a.value, &[])],
                },
                Some(other) => self.err("SX-E08", format!("`grid` takes no option `{other}`; its options are `x`, `y`, `cells`"), a.name.as_ref().unwrap().span),
                None => self.err("SX-E08", "a grid takes named options: `grid rod(x: [0 m, 1 m], cells: 100)`", a.value.span),
            }
        }
        let ranges: Vec<(Expr, Expr)> = xr.into_iter().chain(yr).collect();
        if cells.len() == 1 && ranges.len() == 2 {
            cells.push(cells[0].clone());
        }
        if ranges.is_empty() || cells.len() != ranges.len() {
            self.err("SX-E08", "a grid has an axis and a number of cells, `grid rod(x: [0 m, 1 m], cells: 100)`, or two axes and a number per axis, `grid plate(x: [0 m, 1 m], y: [0 m, 1 m], cells: (50, 50))`", span);
            return;
        }
        let axes = ranges.into_iter().zip(cells).map(|((lo, hi), cells)| GridAxis { lo, hi, cells }).collect();
        self.model.grids.push(Grid { id, name: name.text.clone(), axes });
    }

    /// `boundary T { left = e }`, `boundary T periodic` (MK-9.6, D-091).
    fn boundary(&mut self, field: &ast::Name, periodic: bool, fixed: &[(ast::Name, ast::Expr)]) {
        let id = self.binding(field);
        let fixed: Vec<FixedEdge> = fixed.iter().map(|(e, v)| FixedEdge { edge: e.text.clone(), value: self.expr(v, &[]) }).collect();
        match self.model.bindings.iter_mut().find(|b| b.id == id) {
            Some(b) if b.boundary.is_none() => b.boundary = Some(Boundary { periodic, fixed }),
            Some(_) => self.err("SX-E09", format!("`{}` has two boundary statements", field.text), field.span),
            None => {}
        }
    }

    fn flow(&mut self, f: &ast::FlowStmt, process: Option<&Id>) {
        // `for b in row { der(b.vel) += e }`: the loop variable is in scope (D-055).
        let each = match &f.each {
            Some((var, over)) => match self.part_ref(&over.text).ok() {
                Some(p) => {
                    self.vars.push((var.text.clone(), p.object.clone()));
                    Some(Each { var: var.text.clone(), over: p.id })
                }
                None => {
                    self.err("SX-E03", format!("unknown collection `{}`", over.text), over.span);
                    None
                }
            },
            None => None,
        };
        let (member, target) = match &f.member {
            None => (None, self.binding(&f.target)),
            Some(m) => match self.member_expr(m, &[]) {
                Some((sel, ty)) if !self.objects.contains_key(&ty) => (Some(sel), f.target.text.clone()),
                Some((sel, ty)) => {
                    let id = self.objects.get(&ty).and_then(|o| o.bindings.get(&f.target.text)).cloned();
                    match id {
                        Some(id) => (Some(sel), id),
                        None => {
                            self.err("SX-E03", format!("an object `{ty}` has no binding `{}`", f.target.text), f.target.span);
                            (Some(sel), f.target.text.clone())
                        }
                    }
                }
                None => {
                    self.err("SX-E03", "a flow's target is a binding, or a member's binding: `der(b.vel)`", m.span);
                    (None, f.target.text.clone())
                }
            },
        };
        // `der(T(x)) = e`: a function of the cell's coordinate (D-091).
        let expr = match &f.coord {
            Some(x) => {
                let body = self.expr(&f.expr, std::slice::from_ref(&x.text));
                build::lambda_named(&[x.text.as_str()], vec![Type::real()], body)
            }
            None => self.expr(&f.expr, &[]),
        };
        if each.is_some() {
            self.vars.pop();
        }
        self.flows += 1;
        let id = format!("{}.flow.{}", self.name, self.flows);
        self.map.insert(id.clone(), f.span);
        let kind = if f.contribute { FlowKind::Contribute } else { FlowKind::Define };
        self.model.flows.push(Flow { id, target, kind, expr, process: process.cloned(), member, each });
    }

    fn event(&mut self, e: &ast::EventDecl, process: Option<&Id>) {
        // `for b in drops { event ... }`: the loop variable is in scope (D-057).
        let each = match &e.each {
            Some((var, over)) => match self.part_ref(&over.text).ok() {
                Some(p) => {
                    if !p.many {
                        self.err("SX-E08", format!("`{}` is one object; `for` goes over a collection", over.text), over.span);
                    }
                    self.vars.push((var.text.clone(), p.object.clone()));
                    Some(Each { var: var.text.clone(), over: p.id })
                }
                None => {
                    self.err("SX-E03", format!("unknown collection `{}`", over.text), over.span);
                    None
                }
            },
            None => None,
        };
        let trigger = match &e.trigger {
            ast::TriggerExpr::Rising(g) => Trigger::Rising { guard: self.expr(g, &[]) },
            ast::TriggerExpr::Falling(g) => Trigger::Falling { guard: self.expr(g, &[]) },
            ast::TriggerExpr::Crossing(g) => Trigger::Crossing { guard: self.expr(g, &[]) },
            ast::TriggerExpr::At(t) => Trigger::At { time: self.expr(t, &[]) },
            ast::TriggerExpr::Every(p, from) => Trigger::Every {
                period: self.expr(p, &[]),
                from: match from {
                    Some(f) => self.expr(f, &[]),
                    None => build::t0(),
                },
            },
            ast::TriggerExpr::Start => Trigger::Start,
            ast::TriggerExpr::Input(i) => Trigger::Input { binding: self.binding(i) },
            ast::TriggerExpr::Request(_) => Trigger::Request,
            ast::TriggerExpr::Random(r) => Trigger::Random { rate: self.expr(r, &[]) },
            ast::TriggerExpr::On(n, _) => Trigger::On { event: self.event_id(n) },
        };
        let id = format!("{}.event.{}", self.name, e.name.text);
        // D-050: the payload's name reads the occurrence's payload in the condition and handler;
        // a member payload names a member of its collection, as a loop variable does (D-059).
        let decls: &[ast::PayloadDecl] = match &e.trigger {
            ast::TriggerExpr::Request(d) | ast::TriggerExpr::On(_, d) => d,
            _ => &[],
        };
        let vars = self.vars.len();
        let mut items = vec![];
        for (k, d) in decls.iter().enumerate() {
            let n = &d.name;
            if self.bindings.contains_key(&n.text) || self.functions.contains_key(&n.text) {
                self.err("SX-E09", format!("payload `{}` has the name of a binding or function", n.text), n.span);
            }
            if decls[..k].iter().any(|x| x.name.text == n.text) {
                self.err("SX-E09", format!("payload `{}` declared twice", n.text), n.span);
            }
            let component = if decls.len() > 1 { Some(k) } else { None };
            match &d.ty {
                ast::PayloadTy::Value(t) => {
                    items.push(Payload::value(n.text.clone(), self.ty(t)));
                    self.payload.push((n.text.clone(), id.clone(), component));
                }
                ast::PayloadTy::Member(c) => match self.parts.get(&c.text).cloned() {
                    Some(p) if p.many => {
                        items.push(Payload::member(n.text.clone(), p.id));
                        self.vars.push((n.text.clone(), p.object));
                    }
                    // The name stays a member, of no known type, so that its uses are not
                    // reported again.
                    Some(_) => {
                        self.err("SX-E08", format!("`{}` is one object; a member payload names a member of a collection", c.text), c.span);
                        self.vars.push((n.text.clone(), String::new()));
                    }
                    None => {
                        self.err("SX-E03", format!("unknown collection `{}`", c.text), c.span);
                        self.vars.push((n.text.clone(), String::new()));
                    }
                },
            }
        }
        let payload = match items.len() {
            0 => None,
            1 => items.pop(),
            _ => Some(Payload::several(items)),
        };
        let enable = e.enable.as_ref().map(|c| self.expr(c, &[]));
        let handler = e.handler.iter().map(|o| self.op(o)).collect();
        let zeno = e.zeno.as_ref().map(|z| Zeno {
            policy: match z {
                ast::ZenoClause::Stop => ZenoPolicy::Stop,
                ast::ZenoClause::Settle(ops) => ZenoPolicy::Settle { ops: ops.iter().map(|o| self.op(o)).collect() },
            },
            eps: None,
            n: None,
            window: None,
        });
        self.payload.clear();
        self.vars.truncate(vars);
        if each.is_some() {
            self.vars.pop();
        }
        self.map.insert(id.clone(), e.span);
        self.model.events.push(Event {
            id,
            name: e.name.text.clone(),
            trigger,
            enable,
            handler,
            zeno,
            process: process.cloned(),
            payload,
            each,
            notes: e.notes.clone(),
        });
    }

    pub(crate) fn op(&mut self, o: &ast::OpStmt) -> Op {
        match o {
            ast::OpStmt::Set { target, coord: Some(x), value, .. } => {
                let body = self.expr(value, std::slice::from_ref(&x.text));
                Op::Set { target: self.target(target), value: build::lambda_named(&[x.text.as_str()], vec![Type::real()], body) }
            }
            ast::OpStmt::Set { target, value, .. } => Op::Set { target: self.target(target), value: self.expr(value, &[]) },
            ast::OpStmt::Contribute { target, value, .. } => Op::Contribute { target: self.target(target), value: self.expr(value, &[]) },
            ast::OpStmt::Emit { event, payload, .. } => {
                let payload = payload.as_ref().map(|p| self.expr(p, &[]));
                Op::Emit { event: self.event_id(event), payload }
            }
            // D-057: the overrides name the member's bindings and are read in this scope.
            ast::OpStmt::Create { part, overrides, .. } => {
                // A part path reaches a collection of a contained object (D-074).
                let p = match self.part_ref(&part.text) {
                    Ok(p) => p,
                    Err(m) => {
                        self.err("SX-E03", m, part.span);
                        return Op::Create { part: part.text.clone(), overrides: vec![] };
                    }
                };
                if !p.many {
                    self.err("SX-E08", format!("`{}` is one object; `create` makes a member of a collection", part.text), part.span);
                }
                let obj = self.objects.get(&p.object).cloned().unwrap_or_default();
                let mut ovs = vec![];
                for (n, e) in overrides {
                    let Some(b) = obj.bindings.get(&n.text).cloned() else {
                        self.err("SX-E03", format!("an object `{}` has no binding `{}`", p.object, n.text), n.span);
                        continue;
                    };
                    let value = self.expr(e, &[]);
                    ovs.push(prismal_ir::present::Override { binding: b, value });
                }
                Op::Create { part: p.id, overrides: ovs }
            }
            ast::OpStmt::Destroy { member, span } => match self.member_expr(member, &[]) {
                Some((sel, _)) => Op::Destroy { member: sel },
                None => {
                    self.err("SX-E08", "`destroy` names a member: `destroy b`, `destroy row[2]`", *span);
                    Op::Destroy { member: Self::placeholder() }
                }
            },
            // D-058: endpoints are members, in the order of the relation type's roles.
            ast::OpStmt::Connect { part, ends, overrides, .. } => {
                let p = match self.part_ref(&part.text) {
                    Ok(p) => p,
                    Err(m) => {
                        self.err("SX-E03", m.replace("collection", "set of relations"), part.span);
                        return Op::Connect { part: part.text.clone(), ends: vec![], overrides: vec![] };
                    }
                };
                let obj = self.objects.get(&p.object).cloned().unwrap_or_default();
                if obj.ends.is_empty() && self.objects.contains_key(&p.object) {
                    self.err("SX-E08", format!("`{}` holds objects: `connect` makes a relation, `create` an object", part.text), part.span);
                }
                let mut sels = vec![];
                for e in ends {
                    match self.member_expr(e, &[]) {
                        Some((m, _)) => sels.push(m),
                        None => {
                            self.err("SX-E08", "an endpoint is a member: `connect springs(b, o)`", e.span);
                            sels.push(Self::placeholder());
                        }
                    }
                }
                let mut ovs = vec![];
                for (n, e) in overrides {
                    let Some(b) = obj.bindings.get(&n.text).cloned() else {
                        self.err("SX-E03", format!("a relation `{}` has no binding `{}`", p.object, n.text), n.span);
                        continue;
                    };
                    let value = self.expr(e, &[]);
                    ovs.push(prismal_ir::present::Override { binding: b, value });
                }
                Op::Connect { part: p.id, ends: sels, overrides: ovs }
            }
            ast::OpStmt::Disconnect { relation, span } => match self.member_expr(relation, &[]) {
                Some((sel, _)) => Op::Disconnect { relation: sel },
                None => {
                    self.err("SX-E08", "`disconnect` names a relation: `disconnect s`", *span);
                    Op::Disconnect { relation: Self::placeholder() }
                }
            },
        }
    }

    /// A target of an operation: a binding of the scope, or a binding of a member it
    /// contains (`set b.vel = ...`, D-057). `b.vel` is a member's binding when `b` names a
    /// member and is not a binding.
    pub(crate) fn target(&mut self, p: &ast::Path) -> Target {
        let names_member = |cx: &Self, n: &str| !cx.bindings.contains_key(n) && (cx.vars.iter().any(|(v, _)| v == n) || cx.parts.contains_key(n));
        let (member, name, component) = match (&p.member, &p.component) {
            // `s.b.vel`: the member at the endpoint `b` of `s` (D-059).
            (Some(m), Some(c)) if self.quiet_type(m, &[]).is_some_and(|t| self.objects.get(&t).is_some_and(|o| o.ends.contains_key(&p.name.text))) => {
                (Some(ast::Expr { kind: ExprKind::Field(Box::new(m.clone()), p.name.clone()), span: m.span }), c.clone(), None)
            }
            (Some(m), _) => (Some(m.clone()), p.name.clone(), p.component.clone()),
            (None, Some(c)) if names_member(self, &p.name.text) => {
                (Some(ast::Expr { kind: ExprKind::Name(p.name.text.clone()), span: p.name.span }), c.clone(), None)
            }
            (None, _) => (None, p.name.clone(), p.component.clone()),
        };
        let component = component.as_ref().map(|c| self.axis(c));
        let Some(m) = member else {
            return Target { binding: self.binding(&name), component, member: None };
        };
        match self.member_expr(&m, &[]) {
            Some((sel, ty)) => {
                let binding = match self.objects.get(&ty).and_then(|o| o.bindings.get(&name.text)).cloned() {
                    Some(id) => id,
                    None => {
                        if self.objects.contains_key(&ty) {
                            self.err("SX-E03", format!("an object `{ty}` has no binding `{}`", name.text), name.span);
                        }
                        name.text.clone()
                    }
                };
                Target { binding, component, member: Some(sel) }
            }
            None => {
                self.err("SX-E03", "a target is a binding, or a member's binding: `set b.vel = ...`", m.span);
                Target { binding: name.text.clone(), component, member: None }
            }
        }
    }

    pub(crate) fn binding(&mut self, n: &ast::Name) -> Id {
        match self.bindings.get(&n.text) {
            Some(id) => id.clone(),
            None => {
                self.err("SX-E03", format!("unknown binding `{}` in model `{}`", n.text, self.name), n.span);
                format!("{}.{}", self.name, n.text)
            }
        }
    }

    pub(crate) fn event_id(&mut self, n: &ast::Name) -> Id {
        match self.events.get(&n.text) {
            Some(id) => id.clone(),
            None => {
                self.err("SX-E03", format!("unknown event `{}` in model `{}`", n.text, self.name), n.span);
                format!("{}.event.{}", self.name, n.text)
            }
        }
    }

    fn axis(&mut self, n: &ast::Name) -> usize {
        let axes: Vec<String> = match self.space {
            Some(s) => s.axes.clone(),
            None => ["x", "y", "z"].iter().map(|s| s.to_string()).collect(),
        };
        match axes.iter().position(|a| *a == n.text) {
            Some(i) => i,
            None => {
                self.err("SX-E03", format!("no component `{}`: the axes are {}", n.text, axes.join(", ")), n.span);
                0
            }
        }
    }

    // ------------------------------------------------------------ types

    fn space_id(&mut self, span: Span) -> Id {
        match self.space {
            Some(s) => s.id.clone(),
            None => {
                self.err("SX-E08", format!("model `{}` has no default space: write `model {} in Space`, or name the space", self.name, self.name), span);
                String::new()
            }
        }
    }

    fn dim(&mut self, d: &ast::DimExpr) -> Dim {
        let mut out = Dim::NONE;
        for f in &d.factors {
            match named_dim(&f.name.text) {
                Some(base) => out = out.mul(&base.pow(Ratio::new(f.num, f.den))),
                None => self.err("SX-E04", format!("unknown dimension `{}`", f.name.text), f.name.span),
            }
        }
        out
    }

    fn ty(&mut self, t: &TypeExpr) -> Type {
        let (name, args, span) = match t {
            TypeExpr::Tuple { items, .. } => return Type::Tuple { items: items.iter().map(|i| self.ty(i)).collect() },
            TypeExpr::Named { name, args, span } => (name, args, *span),
        };
        let n = name.text.as_str();
        let arity = |cx: &mut Self, want: &[usize]| {
            if !want.contains(&args.len()) {
                cx.err("SX-E04", format!("`{n}` takes {} type argument(s)", want.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(" or ")), span);
                false
            } else {
                true
            }
        };
        let space_arg = |cx: &mut Self, a: &ast::DimExpr| -> Option<Id> {
            match a.factors.as_slice() {
                [f] if f.num == 1 && f.den == 1 => cx.spaces.iter().find(|s| s.name == f.name.text).map(|s| s.id.clone()),
                _ => None,
            }
        };
        match n {
            "Real" | "Angle" if arity(self, &[0]) => Type::real(),
            "Boolean" if arity(self, &[0]) => Type::Boolean,
            "Integer" if arity(self, &[0]) => Type::Integer,
            "Quantity" if arity(self, &[1]) => Type::Quantity { dim: self.dim(&args[0]) },
            "Instant" if arity(self, &[1]) => Type::Instant { dim: self.dim(&args[0]) },
            "Point" if arity(self, &[0, 1]) => match args.first() {
                None => Type::Point { space: self.space_id(span) },
                Some(a) => match space_arg(self, a) {
                    Some(s) => Type::Point { space: s },
                    None => {
                        self.err("SX-E03", "unknown space in `Point<...>`", a.span);
                        Type::Point { space: String::new() }
                    }
                },
            },
            "Vector" if arity(self, &[1, 2]) => {
                if args.len() == 2 {
                    let space = match space_arg(self, &args[0]) {
                        Some(s) => s,
                        None => {
                            self.err("SX-E03", "unknown space in `Vector<Space, D>`", args[0].span);
                            String::new()
                        }
                    };
                    Type::Vector { space, dim: self.dim(&args[1]) }
                } else {
                    let space = self.space_id(span);
                    Type::Vector { space, dim: self.dim(&args[0]) }
                }
            }
            "Real" | "Angle" | "Boolean" | "Integer" | "Quantity" | "Instant" | "Point" | "Vector" => Type::real(),
            _ if args.is_empty() && self.enums.contains_key(n) => {
                let (id, cases) = self.enums[n].clone();
                Type::Enum { r#enum: id, cases }
            }
            _ => match named_dim(n) {
                Some(dim) if args.is_empty() => Type::Quantity { dim },
                _ => {
                    self.err("SX-E04", format!("unknown type `{n}`"), name.span);
                    Type::real()
                }
            },
        }
    }

    // ------------------------------------------------------------ expressions

    fn interval(&mut self, x: Expr, i: &ast::Interval, locals: &[String]) -> Expr {
        let lo = match &i.lo {
            Bound::Inf => None,
            Bound::Value(e) => Some(self.expr(e, locals)),
        };
        let hi = match &i.hi {
            Bound::Inf => None,
            Bound::Value(e) => Some(self.expr(e, locals)),
        };
        let lo_cmp = |lo: Expr| if i.lo_closed { build::le(lo, x.clone()) } else { build::lt(lo, x.clone()) };
        let hi_cmp = |hi: Expr| if i.hi_closed { build::le(x.clone(), hi) } else { build::lt(x.clone(), hi) };
        match (lo, hi) {
            (Some(l), Some(h)) => build::and(lo_cmp(l), hi_cmp(h)),
            // One-sided: the value on the left, as `where x >= lo` writes it, so that both
            // spellings give one IR (working syntax section 1.4).
            (Some(l), None) => {
                if i.lo_closed {
                    build::ge(x.clone(), l)
                } else {
                    build::gt(x.clone(), l)
                }
            }
            (None, Some(h)) => hi_cmp(h),
            (None, None) => {
                self.err("SX-E08", "an interval needs at least one finite bound", i.span);
                build::boolean(true)
            }
        }
    }

    fn placeholder() -> Expr {
        build::lit(0.0)
    }

    pub(crate) fn expr(&mut self, e: &ast::Expr, locals: &[String]) -> Expr {
        use ast::BinOp as B;
        match &e.kind {
            ExprKind::Num(v, unit) => match unit {
                None => build::lit(*v),
                Some(u) => match Unit::parse(&u.text) {
                    Ok(unit) => Expr::Num { num: *v, unit: Some(unit) },
                    Err(msg) => {
                        self.err("SX-E05", format!("{msg} (in a model)"), u.span);
                        Self::placeholder()
                    }
                },
            },
            ExprKind::Bool(b) => build::boolean(*b),
            ExprKind::Name(n) => self.name(n, e.span, locals),
            // A minus sign on a number literal writes a negative literal (IR-1.1).
            ExprKind::Neg(x) => match self.expr(x, locals) {
                Expr::Num { num, unit } if matches!(x.kind, ExprKind::Num(..)) => Expr::Num { num: -num, unit },
                e => -e,
            },
            ExprKind::Not(x) => build::not(self.expr(x, locals)),
            ExprKind::Norm(x) => build::norm(self.expr(x, locals)),
            ExprKind::Binary(op, l, r) => {
                let (l, r) = (self.expr(l, locals), self.expr(r, locals));
                let op = match op {
                    B::Add => BinOp::Add,
                    B::Sub => BinOp::Sub,
                    B::Mul => BinOp::Mul,
                    B::Div => BinOp::Div,
                    B::Pow => BinOp::Pow,
                    B::And => BinOp::And,
                    B::Or => BinOp::Or,
                };
                Expr::bin(op, l, r)
            }
            ExprKind::Compare(first, rest) => {
                // Chained comparisons lower to `and` of pairwise comparisons (04-ir section 6).
                let mut left = self.expr(first, locals);
                let mut out: Option<Expr> = None;
                for (op, r) in rest {
                    let right = self.expr(r, locals);
                    let op = match op {
                        ast::CmpOp::Eq => BinOp::Eq,
                        ast::CmpOp::Ne => BinOp::Ne,
                        ast::CmpOp::Lt => BinOp::Lt,
                        ast::CmpOp::Le => BinOp::Le,
                        ast::CmpOp::Gt => BinOp::Gt,
                        ast::CmpOp::Ge => BinOp::Ge,
                    };
                    let c = Expr::bin(op, left, right.clone());
                    out = Some(match out {
                        None => c,
                        Some(prev) => build::and(prev, c),
                    });
                    left = right;
                }
                out.expect("a comparison has at least one operator")
            }
            ExprKind::In(x, i) => {
                let x = self.expr(x, locals);
                self.interval(x, i, locals)
            }
            ExprKind::If(c, a, b) => build::ite(self.expr(c, locals), self.expr(a, locals), self.expr(b, locals)),
            ExprKind::Otherwise(a, b) => Expr::Otherwise { otherwise: Box::new(self.expr(a, locals)), default: Box::new(self.expr(b, locals)) },
            ExprKind::Call(..) if self.rel_method(e, locals).is_some() => {
                let (method, rel, of) = self.rel_method(e, locals).unwrap();
                match method {
                    "has" => match (self.member_expr(rel, locals), self.member_expr(of, locals)) {
                        (Some((rel, _)), Some((of, _))) => Expr::Has { has: Box::new(of), rel: Box::new(rel) },
                        _ => Self::placeholder(),
                    },
                    // A member, compared with `==` or read by a binding (D-064).
                    _ => self.member_expr(e, locals).map(|(m, _)| m).unwrap_or_else(Self::placeholder),
                }
            }
            ExprKind::Call(f, args) => self.call(f, args, e.span, locals),
            ExprKind::Field(x, n) if matches!(&x.kind, ExprKind::Name(e) if self.enums.contains_key(e) && !self.bindings.contains_key(e) && !locals.contains(e)) => {
                let ExprKind::Name(e) = &x.kind else { unreachable!() };
                if !self.enums[e].1.contains(&n.text) {
                    self.err("SX-E03", format!("`{}` is not a case of `{e}`", n.text), n.span);
                }
                Expr::Case { case: n.text.clone() }
            }
            ExprKind::Field(x, n) => match self.member_expr(x, locals) {
                // An endpoint of a relation: `s.a` (D-058).
                Some((of, ty)) if self.objects.get(&ty).is_some_and(|o| o.ends.contains_key(&n.text)) => {
                    Expr::End { end: self.objects[&ty].ends[&n.text].0.clone(), of: Some(Box::new(of)) }
                }
                // A member's binding: `b.pos`, `ball.pos`, `row[2].pos` (D-055). A part of
                // an unknown type is reported where it is declared, not at each use.
                Some((_, ty)) if !self.objects.contains_key(&ty) => Self::placeholder(),
                Some((of, ty)) => match self.objects.get(&ty).and_then(|o| o.bindings.get(&n.text)).cloned() {
                    Some(field) => Expr::Field { field, of: Box::new(of) },
                    None => {
                        self.err("SX-E03", format!("an object `{ty}` has no binding `{}`", n.text), n.span);
                        Self::placeholder()
                    }
                },
                None => {
                    // A mistake inside is reported once, not again as a missing component.
                    let before = self.diags.len();
                    let x = self.expr(x, locals);
                    if self.diags.len() > before {
                        return Self::placeholder();
                    }
                    let axis = self.axis(n);
                    build::comp(x, axis)
                }
            },
            ExprKind::Tuple(items) => build::tuple(items.iter().map(|i| self.expr(i, locals)).collect()),
            ExprKind::Str(_) => {
                self.err("SX-E08", "text belongs to presentations, not to a model", e.span);
                Self::placeholder()
            }
            ExprKind::Index(..) => match self.member_expr(e, locals) {
                Some((m, _)) => m,
                None => {
                    self.err("SX-E08", "only a collection is indexed: `row[2]`", e.span);
                    Self::placeholder()
                }
            },
            ExprKind::Aggregate { .. } => {
                self.err("SX-E08", "an aggregate is written as the argument of `sum`, `min`, `max`, `any`, `all` or `count`", e.span);
                Self::placeholder()
            }
            ExprKind::Match(x, arms) => {
                let scrutinee = self.expr(x, locals);
                let arms = arms.iter().map(|(c, v)| Arm { case: c.text.clone(), value: self.expr(v, locals) }).collect();
                Expr::Match { r#match: Box::new(scrutinee), arms }
            }
            ExprKind::List(_) | ExprKind::Map(..) | ExprKind::On(..) | ExprKind::BeatTime { .. } => {
                self.err("SX-E08", "this form belongs to presentations and runs, not to a model", e.span);
                Self::placeholder()
            }
        }
    }

    fn name(&mut self, n: &str, span: Span, locals: &[String]) -> Expr {
        if let Some(i) = locals.iter().position(|l| l == n) {
            return build::param(i);
        }
        if let Some((_, ev, k)) = self.payload.iter().find(|(p, ..)| p == n) {
            let read = Expr::Payload { payload: ev.clone() };
            return match k {
                Some(k) => Expr::Comp { comp: Box::new(read), axis: *k },
                None => read,
            };
        }
        if let Some(id) = self.bindings.get(n) {
            return build::r(id);
        }
        // Members of objects (D-055): a loop variable, a contained object.
        if let Some((_, _)) = self.vars.iter().rev().find(|(v, _)| v == n) {
            return Expr::Var { var: n.to_string() };
        }
        // An endpoint inside its relation type (D-058).
        if let Some((end, _)) = self.ends.get(n) {
            return Expr::End { end: end.clone(), of: None };
        }
        if let Some(p) = self.parts.get(n) {
            if !p.many {
                return Expr::Part { part: p.id.clone() };
            }
        }
        if n == "index" && self.index_ok {
            return Expr::Builtin { builtin: Builtin::Index };
        }
        if let Some(id) = self.functions.get(n) {
            return Expr::Fn { r#fn: id.clone() };
        }
        // A case of a declared enumeration; its type comes from the context (D-049).
        if self.enums.values().any(|(_, cases)| cases.iter().any(|c| c == n)) {
            return Expr::Case { case: n.to_string() };
        }
        match n {
            "t" => build::t(),
            "t0" => build::t0(),
            "elapsed" => build::elapsed(),
            // White noise in flows (D-090); a name the author declares wins.
            "noise" => Expr::Builtin { builtin: Builtin::Noise },
            "π" | "pi" => build::pi(),
            "origin" => {
                let s = self.space_id(span);
                build::origin(&s)
            }
            "inf" => {
                self.err("SX-E08", "`inf` is only an interval bound", span);
                Self::placeholder()
            }
            _ => {
                let hint = if self.events.contains_key(n) { " (an event is not a value)" } else { "" };
                self.err("SX-E03", format!("unknown name `{n}` in model `{}`{hint}", self.name), span);
                Self::placeholder()
            }
        }
    }

    fn call(&mut self, f: &ast::Expr, args: &[ast::Arg], span: Span, locals: &[String]) -> Expr {
        for a in args {
            if let Some(n) = &a.name {
                self.err("SX-E08", "named arguments belong to representations, not to model expressions", n.span);
            }
            if let Some(e) = &a.every {
                self.err("SX-E08", "a sampled source (`every`) belongs to a representation, not to a model expression", e.span);
            }
            if let Some(e) = &a.on {
                self.err("SX-E08", "values at an event's occurrences (`on E`) belong to a representation, not to a model expression", e.span);
            }
        }
        let name = match &f.kind {
            ExprKind::Name(n) => n.as_str(),
            _ => {
                let callee = self.expr(f, locals);
                let args = args.iter().map(|a| self.expr(&a.value, locals)).collect();
                return build::apply(callee, args);
            }
        };
        if name == "der" && !locals.iter().any(|l| l == name) && !self.bindings.contains_key(name) {
            return match args {
                [ast::Arg { value: ast::Expr { kind: ExprKind::Name(x), span: xs }, .. }] => {
                    let id = self.binding(&ast::Name { text: x.clone(), span: *xs });
                    Expr::Der { der: id }
                }
                _ => {
                    self.err("SX-E08", "`der` takes the name of one continuous state", span);
                    Self::placeholder()
                }
            };
        }
        if !locals.iter().any(|l| l == name) && !self.bindings.contains_key(name) && !self.functions.contains_key(name) {
            if let Some(a) = self.aggregate(name, args, span, locals) {
                return a;
            }
        }
        let lowered: Vec<Expr> = args.iter().map(|a| self.expr(&a.value, locals)).collect();
        if locals.iter().any(|l| l == name) || self.bindings.contains_key(name) || self.functions.contains_key(name) {
            let callee = self.name(name, f.span, locals);
            return build::apply(callee, lowered);
        }
        if let Some((_, func, n)) = FUNCS.iter().find(|(s, _, _)| *s == name) {
            let one_field = matches!(func, Func::Min | Func::Max) && lowered.len() == 1;
            if lowered.len() != *n && !one_field {
                self.err("SX-E08", format!("`{name}` takes {n} argument(s), found {}", lowered.len()), span);
            }
            return build::call(*func, lowered);
        }
        // Distributions are drawn from in an event's condition and operations (D-083).
        if let Some(d) = prismal_ir::Dist::from_name(name) {
            return Expr::Draw { draw: d, args: lowered };
        }
        self.err("SX-E03", format!("unknown function `{name}`"), f.span);
        Self::placeholder()
    }
}

/// The names of a lowered object type or model: its bindings and parts (D-055).
pub(crate) fn obj_info(m: &Model) -> ObjInfo {
    ObjInfo {
        id: m.id.clone(),
        bindings: m.bindings.iter().map(|b| (b.name.clone(), b.id.clone())).collect(),
        parts: m
            .parts
            .iter()
            .map(|p| {
                let object = p.object.rsplit('.').next().unwrap_or(&p.object).to_string();
                (p.name.clone(), PartInfo { id: p.id.clone(), object, many: p.count.is_some() || p.capacity.is_some() || p.unbounded })
            })
            .collect(),
        ends: HashMap::new(),
    }
}
