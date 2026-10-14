//! Prismal semantic IR (docs/spec/04-ir.md).
//!
//! The IR records what the author meant: resolved identities, roles, types, flow kinds and
//! trigger directions (R-47). It is serialized as versioned JSON (D-037).

pub mod build;
pub mod dim;
pub mod expr;
pub mod present;
pub mod units;

pub use dim::{Dim, Ratio};
pub mod elaborate;
pub mod enforce;
pub mod link;

pub use expr::{Agg, Arm, BinOp, Builtin, Constant, Dist, Expr, Func, Guarded, Lambda};
pub use units::Unit;

use serde::{Deserialize, Serialize};

/// Element identity (IR-1.4): opaque, unique within a document, stable across edits.
pub type Id = String;

pub const FORMAT: &str = "prismal-ir";
pub const VERSION: &str = "0.1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub format: String,
    pub version: String,
    #[serde(default)]
    pub spaces: Vec<Space>,
    #[serde(default)]
    pub models: Vec<Model>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub presentations: Vec<present::Presentation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<present::RunCase>,
    /// The modules the document uses, `use name` or `use name.{A, B}` (D-099).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uses: Vec<Use>,
    /// Definitions written outside models (D-099): the document's own, as the library named
    /// `SHARED`, and those of every module it uses, directly or through another module, so
    /// that the document is complete without its modules' files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<Library>,
}

/// `use module` or `use module.{A, B}` (D-099): the module's shared definitions, or the named
/// ones, become visible to the document's models by name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Use {
    pub module: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
}

/// Object types, enumerations and functions written outside any model (MK-7.1a, D-099). They
/// are written once for every space: their points and vectors are of the space `ANY_SPACE`,
/// which becomes the space of the model that uses them (`link`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Library {
    /// The module's name, or `SHARED` for the document's own definitions.
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<EnumDecl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<FunctionDecl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<Model>,
}

/// The library of a document's own shared definitions (D-099).
pub const SHARED: &str = "shared";
/// The space of shared definitions: the space of the model that uses them (D-099).
pub const ANY_SPACE: &str = "@space";

impl Document {
    pub fn new(spaces: Vec<Space>, models: Vec<Model>) -> Document {
        Document { format: FORMAT.into(), version: VERSION.into(), spaces, models, presentations: vec![], runs: vec![], uses: vec![], libraries: vec![] }
    }
    pub fn model(&self, id: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.id == id)
    }
    pub fn presentation(&self, id: &str) -> Option<&present::Presentation> {
        self.presentations.iter().find(|p| p.id == id)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("IR serializes")
    }
    pub fn from_json(s: &str) -> Result<Document, String> {
        let d: Document = serde_json::from_str(s).map_err(|e| e.to_string())?;
        if d.format != FORMAT {
            return Err(format!("not a Prismal IR document (format `{}`)", d.format));
        }
        Ok(d)
    }
}

/// Types (MK section 2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Type {
    Boolean,
    Integer,
    /// `Real` is the dimensionless quantity (MK-3.2).
    Quantity { dim: Dim },
    Instant { dim: Dim },
    Point { space: Id },
    Vector { space: Id, dim: Dim },
    Tuple { items: Vec<Type> },
    /// A declared enumeration (D-049): its identity, which makes it nominal (MK-2.2), and
    /// its cases in declaration order.
    Enum { r#enum: Id, cases: Vec<String> },
    Function { params: Vec<Type>, result: Box<Type> },
    /// A field over a grid of the model (MK-9.4, D-091): a value of type `of` in every cell.
    Field { grid: Id, of: Box<Type> },
}

impl Type {
    pub fn real() -> Type {
        Type::Quantity { dim: Dim::NONE }
    }
    pub fn qty(dim: &str) -> Type {
        Type::Quantity { dim: Dim::parse(dim).expect("valid dimension") }
    }
    pub fn point(space: &str) -> Type {
        Type::Point { space: space.into() }
    }
    pub fn vector(space: &str, dim: &str) -> Type {
        Type::Vector { space: space.into(), dim: Dim::parse(dim).expect("valid dimension") }
    }
    pub fn func(params: Vec<Type>, result: Type) -> Type {
        Type::Function { params, result: Box::new(result) }
    }
}

/// A space (MK-4.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Space {
    pub id: Id,
    pub name: String,
    pub dimension: u32,
    pub axes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Constant,
    Parameter,
    Input,
    Discrete,
    Continuous,
    Derived,
    /// An algebraic unknown (MK-11.6, D-092): found, at every instant, from the equations
    /// that read it. Its `init` is the starting guess of the solver.
    Unknown,
}

impl Role {
    pub fn is_stored(self) -> bool {
        !matches!(self, Role::Derived | Role::Unknown)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Display {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// A binding (MK-6.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub id: Id,
    pub name: String,
    pub role: Role,
    #[serde(rename = "type")]
    pub ty: Type,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub def: Option<Expr>,
    /// Explicit intervenability; `None` means the role's default (MK-6.10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intervenable: Option<bool>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub private: bool,
    #[serde(default)]
    pub display: Display,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// A derived binding of a member or relation that may not be alive is evaluated only
    /// while this holds; otherwise it has no value (D-058). Made by elaboration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Expr>,
    /// How `contribute` operations combine into this discrete binding: `sum`, `product`,
    /// `min`, `max`, `any` or `all` (MK-14.9, D-073).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combine: Option<String>,
    /// The boundary conditions of a field state (MK-9.6, D-091); edges not named are
    /// insulated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<Boundary>,
}

/// Boundary conditions of a field (MK-9.6, D-091): `periodic`, or edges held at values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Boundary {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub periodic: bool,
    /// `left = e`: the field is held at `e` on that edge (`left`, `right`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<FixedEdge>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixedEdge {
    pub edge: String,
    pub value: Expr,
}

/// A grid (MK-9.5, D-091): the cells over which field state is stored, one axis per
/// dimension, each an interval of a quantity cut into equal cells.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub id: Id,
    pub name: String,
    pub axes: Vec<GridAxis>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridAxis {
    pub lo: Expr,
    pub hi: Expr,
    pub cells: Expr,
}

impl Binding {
    /// MK-6.10: parameters by default, state only when declared, never others.
    pub fn is_intervenable(&self) -> bool {
        match self.role {
            Role::Parameter => self.intervenable.unwrap_or(true),
            Role::Discrete | Role::Continuous => self.intervenable.unwrap_or(false),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessKind {
    Continuous,
    Discrete,
    Algorithmic,
    Stochastic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Process {
    pub id: Id,
    pub name: String,
    pub kind: ProcessKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowKind {
    Define,
    Contribute,
}

/// A flow `der(target) = expr` or `der(target) += expr` (MK-14.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub id: Id,
    pub target: Id,
    pub kind: FlowKind,
    pub expr: Expr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<Id>,
    /// The member whose binding `target` is, when the flow is written by a container for a
    /// contained object: `der(ball.vel)` (`{"part": ...}`), or `der(b.vel)` in a loop
    /// (`{"var": "b"}` with `each`) (D-055).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<Expr>,
    /// `for b in row { ... }`: the flow is repeated for each member of `over`, named `var`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub each: Option<Each>,
}

/// A loop over the members of a collection (D-055).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Each {
    pub var: String,
    pub over: Id,
}

/// A contained object or a collection (MK-7.11, MK-8.1, D-055): `count` members (one when
/// absent: a contained object) of the object type `object`, each with the overrides, read in
/// the container's scope, where `{"builtin": "index"}` is its number. With `capacity`, the
/// membership changes during a run (D-057): `count` members at the start (none when absent),
/// more made by `create`, removed by `destroy`, and at most `capacity` made in one run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub id: Id,
    pub name: String,
    pub object: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    /// A collection whose membership changes without a declared limit, `Drop[max inf]`
    /// (D-066): `capacity` is absent and the runtime chooses one, growing it as runs need.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unbounded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<present::Override>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// Event triggers (MK-15.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Trigger {
    Rising { guard: Expr },
    Falling { guard: Expr },
    Crossing { guard: Expr },
    At { time: Expr },
    Every { period: Expr, from: Expr },
    On { event: Id },
    Start,
    Input { binding: Id },
    Request,
    /// The jumps of a Poisson process of rate `rate` (D-083, MK-15.15).
    Random { rate: Expr },
    /// Invalid in a checked model (MK-E13); exists so that such IR can be represented and rejected.
    Level { cond: Expr },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub binding: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<usize>,
    /// The member whose binding `binding` is, when a container's handler writes it:
    /// `set b.vel = ...` (`{"var": "b"}`), `set ball.vel = ...` (D-057, MK-7.10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<Expr>,
}

impl Target {
    pub fn of(binding: impl Into<Id>) -> Target {
        Target { binding: binding.into(), component: None, member: None }
    }
}

/// Operations (MK section 16). `create` and `destroy` act on collections whose membership
/// changes (D-057); elaboration turns them into `set` and `destroy` on liveness bindings and
/// `if`; `connect` and `disconnect` are reserved with relations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Set { target: Target, value: Expr },
    Contribute { target: Target, value: Expr },
    Emit {
        event: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Expr>,
    },
    /// `create drops { pos = p }`: a new member of the collection `part`, its stored
    /// bindings starting at the overrides, read in the handler's scope (MK section 16).
    Create {
        part: Id,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        overrides: Vec<present::Override>,
    },
    /// `destroy b`: the member leaves its collection. After elaboration `member` is a
    /// reference to the member's liveness binding (D-057); destroying a member twice in one
    /// transition is not a conflict (MK-16.5).
    Destroy { member: Expr },
    /// `connect springs(x, y) { k = e }`: a new relation instance of the relation set `part`,
    /// with endpoints `ends` (members, in the order of the relation type's roles) and
    /// starting values (MK-8.6, D-058).
    Connect {
        part: Id,
        ends: Vec<Expr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        overrides: Vec<present::Override>,
    },
    /// `disconnect s`: the relation instance leaves its set (D-058).
    Disconnect { relation: Expr },
    /// A member being made: its liveness binding `alive` becomes true and its stored
    /// bindings, parameters included, take their starting values (MK-16.1a). Made by
    /// elaboration of `create` and `connect` (D-057, D-058).
    Make {
        alive: Id,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        values: Vec<present::Override>,
    },
    /// The operations `then`, performed only when `if` holds on the state before the
    /// transition. Made by elaboration (D-057); conflicts count only operations performed.
    If {
        r#if: Expr,
        then: Vec<Op>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum ZenoPolicy {
    Stop,
    Settle { ops: Vec<Op> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zeno {
    #[serde(flatten)]
    pub policy: ZenoPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<f64>,
}

/// An event (MK section 15).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: Id,
    pub name: String,
    pub trigger: Trigger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable: Option<Expr>,
    #[serde(default)]
    pub handler: Vec<Op>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zeno: Option<Zeno>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<Id>,
    /// The value each occurrence carries (MK-15.1, D-050), read in the enabling condition
    /// and the handler as `{"payload": id}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    /// `for b in drops { event ... }`: a container's event repeated for each member of a
    /// collection, named `var` in its trigger, condition and handler (D-057).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub each: Option<Each>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// An event's payload: its name as written (for display) and its type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
    /// A member of the collection with this part identity (`on request(b in balls)`, D-059):
    /// the payload is the member's number in its collection, of type `Real`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<Id>,
    /// Made by elaboration for a member payload: the collection's path in the flat model
    /// (`balls`, `cart.wheels`), by which hosts name a member (`balls[2]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<String>,
    /// Several payloads (`on request(b in balls, j: Momentum)`, D-059): the payload is the
    /// tuple of these, each read as its component; `name` lists their names and `ty` is the
    /// tuple of their types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Payload>,
}

impl Payload {
    /// A payload of a value of type `ty`.
    pub fn value(name: impl Into<String>, ty: Type) -> Payload {
        Payload { name: name.into(), ty, of: None, members: None, items: vec![] }
    }
    /// A member of the collection `of`, by its number.
    pub fn member(name: impl Into<String>, of: impl Into<Id>) -> Payload {
        Payload { name: name.into(), ty: Type::real(), of: Some(of.into()), members: None, items: vec![] }
    }
    /// Several payloads, carried as one tuple.
    pub fn several(items: Vec<Payload>) -> Payload {
        let name = items.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
        let ty = Type::Tuple { items: items.iter().map(|p| p.ty.clone()).collect() };
        Payload { name, ty, of: None, members: None, items }
    }
    /// The payloads as declared: the items, or this one.
    pub fn declared(&self) -> Vec<&Payload> {
        if self.items.is_empty() {
            vec![self]
        } else {
            self.items.iter().collect()
        }
    }
    /// Whether two events carry the same kind of payload: types and collections (D-059).
    pub fn same_kind(&self, other: &Payload) -> bool {
        let (a, b) = (self.declared(), other.declared());
        self.ty == other.ty && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.ty == y.ty && x.of == y.of)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquationRole {
    Display,
    Check,
}

/// An equation (MK section 11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equation {
    pub id: Id,
    pub name: String,
    pub lhs: Expr,
    pub rhs: Expr,
    pub role: EquationRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tol: Option<Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    Reject,
    Report,
    Stop,
    /// The runtime keeps the constraint satisfied (MK-12.6, D-093): the state is projected
    /// onto it, and the unknowns that hold it are solved from its rates.
    Enforce,
}

/// A constraint (MK section 12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Id,
    pub name: String,
    pub cond: Expr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tol: Option<Expr>,
    pub policy: Policy,
    /// Printing hint only (04-ir section 5.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<Id>,
    /// The unknown that holds an enforced inequality constraint, `with N` (MK-12.11, D-098):
    /// positive while it acts, zero while the constraint is not in contact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<Id>,
}

/// A model: the root object type (MK-7.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: Id,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_space: Option<Id>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
    #[serde(default)]
    pub processes: Vec<Process>,
    #[serde(default)]
    pub flows: Vec<Flow>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub equations: Vec<Equation>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Declared enumerations (MK-2.2, D-049).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<EnumDecl>,
    /// Grids of field state (MK-9.5, D-091).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grids: Vec<Grid>,
    /// Declared functions (MK-10.3, D-048).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<FunctionDecl>,
    /// Object types declared in this model (MK-7.1, D-055), each with the body of a model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<Model>,
    /// Contained objects and collections (MK-7.11, MK-8.1, D-055).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<Part>,
    /// The endpoint roles of a relation type (MK-8.5, D-058); empty for a model or an object
    /// type. A part whose type has ends is a relation set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ends: Vec<End>,
    /// An undirected relation type (MK-8.5b, D-064): its two endpoints, in one collection,
    /// have no order outside its body.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub undirected: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// An endpoint role of a relation type (MK-8.5, D-058): its identity (`Model.Spring.a`), its
/// name, and the part of the containing model its endpoint is a member of.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct End {
    pub id: Id,
    pub name: String,
    /// The collection: a part of the containing model, or a part path through contained
    /// objects, `a in left.atoms` (D-065).
    pub over: Id,
}

/// The separator of a part path: the identities of contained objects, then of a part of the
/// last of them (`Cells.part.left/Cells.Cell.part.atoms`, D-065).
pub const PART_PATH: char = '/';

/// A declared enumeration (MK-2.2, D-049): a nominal type whose values are its cases. Its
/// identity is `Model.enum.Name`; types that use it carry that identity and the cases.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnumDecl {
    pub id: Id,
    pub name: String,
    pub cases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl EnumDecl {
    /// The type whose values are this enumeration's cases.
    pub fn ty(&self) -> Type {
        Type::Enum { r#enum: self.id.clone(), cases: self.cases.clone() }
    }
}

/// A declared function (MK-10.3, D-048): typed parameters, a result type and a body that
/// reads only its parameters (`{"param": i}`), constants and other declared functions. Its
/// identity is `Model.fn.name`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub id: Id,
    pub name: String,
    pub params: Vec<FnParam>,
    pub result: Type,
    pub body: Expr,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl FunctionDecl {
    /// The function's type, `(A, B) -> R`.
    pub fn ty(&self) -> Type {
        Type::func(self.params.iter().map(|p| p.ty.clone()).collect(), self.result.clone())
    }
    pub fn param_names(&self) -> Vec<String> {
        self.params.iter().map(|p| p.name.clone()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

impl Model {
    /// Whether the model contains objects, so that it needs elaboration (D-055).
    pub fn has_parts(&self) -> bool {
        !self.parts.is_empty()
    }
    pub fn object(&self, id: &str) -> Option<&Model> {
        self.objects.iter().find(|o| o.id == id)
    }
    pub fn part(&self, id: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == id)
    }
    pub fn function(&self, id: &str) -> Option<&FunctionDecl> {
        self.functions.iter().find(|f| f.id == id)
    }
    pub fn enum_decl(&self, id: &str) -> Option<&EnumDecl> {
        self.enums.iter().find(|e| e.id == id)
    }
    pub fn binding(&self, id: &str) -> Option<&Binding> {
        self.bindings.iter().find(|b| b.id == id)
    }
    pub fn binding_by_name(&self, name: &str) -> Option<&Binding> {
        self.bindings.iter().find(|b| b.name == name)
    }
    pub fn event_by_name(&self, name: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.name == name)
    }
}
