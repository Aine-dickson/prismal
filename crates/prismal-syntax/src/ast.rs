//! Syntax tree of the working syntax. It records what was written, including forms the IR
//! does not distinguish (IR-1.1); lowering (`lower.rs`) turns it into the IR.

use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct File {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Space(SpaceDecl),
    Model(ModelDecl),
    Presentation(PresentationDecl),
    Run(RunDecl),
    /// `use name` or `use name.{A, B}` (D-099).
    Use(UseDecl),
    /// An object type, enumeration or function written outside models (D-099).
    Shared(Member),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UseDecl {
    /// The module's name, its parts as written: `physics.mechanics`.
    pub path: Vec<Name>,
    /// The names imported, or every shared definition of the module.
    pub names: Option<Vec<Name>>,
    pub span: Span,
}

// ---------------------------------------------------------------- models

#[derive(Clone, Debug, PartialEq)]
pub struct SpaceDecl {
    pub name: Name,
    pub kind: Name,
    pub args: Vec<Arg>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelDecl {
    pub name: Name,
    pub space: Option<Name>,
    pub members: Vec<Member>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Member {
    Decl(Decl),
    Fn(FnDecl),
    Enum(EnumDecl),
    Flow(FlowStmt),
    Process(ProcessDecl),
    Event(EventDecl),
    Equation(EquationDecl),
    Constraint(ConstraintDecl),
    /// `object Name { ... }`: an object type (MK section 7, D-055).
    Object(ObjectDecl),
    /// `parts { ball: Ball { ... }  row: Ball[3] { ... } }` (D-055).
    Parts(Vec<PartDecl>),
    /// `grid rod(x: [0 m, 1 m], cells: 100)` (MK-9.5, D-091).
    Grid { name: Name, args: Vec<Arg>, span: Span },
    /// `frame turning(at: P, x: u)`, `frame cabin(at: P, x: u, y: v)` (MK-4.2, D-105).
    Frame { name: Name, args: Vec<Arg>, span: Span },
    /// `boundary T { left = e; right = e }` or `boundary T periodic` (MK-9.6, D-091).
    Boundary { field: Name, periodic: bool, fixed: Vec<(Name, Expr)>, span: Span },
}

/// An object type: the body of a model, declared in one (D-055). A relation type has
/// endpoint roles, each a member of a collection of the model: `relation Spring(a in balls,
/// b in balls) { ... }` (D-058).
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDecl {
    pub name: Name,
    /// A relation type's endpoints and their collections, a path through contained objects
    /// (`a in left.atoms`, D-065).
    pub ends: Vec<(Name, Vec<Name>)>,
    /// `undirected relation ...` (D-064).
    pub undirected: bool,
    pub members: Vec<Member>,
    pub notes: Vec<String>,
    pub span: Span,
}

/// A contained object (`ball: Ball`) or a collection of `count` members (`row: Ball[3]`),
/// with overrides of its members' bindings (D-055). A collection whose membership changes
/// has a capacity: `drops: Drop[max 50]`, `drops: Drop[3, max 50]` (D-057).
#[derive(Clone, Debug, PartialEq)]
pub struct PartDecl {
    pub name: Name,
    pub object: Name,
    pub count: Option<u32>,
    pub capacity: Option<u32>,
    /// `[max inf]`: membership changes without a declared limit (D-066).
    pub unbounded: bool,
    pub overrides: Vec<(Name, Expr)>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleWord {
    Const,
    Param,
    Input,
    State,
    Discrete,
    Derived,
    /// `unknown { I: Current = 0 A }` (D-092).
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decl {
    pub role: RoleWord,
    pub name: Name,
    /// Parameters of a derived function value: `f(x: Real): Real = e` (MK-10.7).
    pub params: Option<Vec<(Name, TypeExpr)>>,
    /// A field: `T(x in rod)`, its coordinate and grid (MK-9.4, D-091).
    pub over: Option<(Name, Name)>,
    pub ty: TypeExpr,
    pub value: Option<Expr>,
    pub range: Option<Range>,
    pub modifiers: Vec<Modifier>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FnDecl {
    pub name: Name,
    pub params: Vec<(Name, TypeExpr)>,
    pub result: TypeExpr,
    pub body: Expr,
    pub notes: Vec<String>,
    pub span: Span,
}

/// `enum Phase { rising, falling, resting }` (MK-2.2, D-049).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumDecl {
    pub name: Name,
    pub cases: Vec<Name>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Range {
    In(Interval),
    Where(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Modifier {
    Intervenable,
    Private,
    Symbol(String),
    Unit(UnitExpr),
    /// `combine sum` (MK-14.9, D-073).
    Combine(Name),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowStmt {
    pub target: Name,
    /// The member whose binding `target` is: `b`, `ball`, `row[2]` in `der(b.vel)` (D-055).
    pub member: Option<Expr>,
    /// `for b in row { ... }`: the loop variable and the collection.
    pub each: Option<(Name, Name)>,
    /// `der(T(x))`: the coordinate of a field's flow (D-091).
    pub coord: Option<Name>,
    pub contribute: bool,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessDecl {
    pub name: Name,
    pub flows: Vec<FlowStmt>,
    pub events: Vec<EventDecl>,
    pub notes: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventDecl {
    /// `for b in drops { event ... }`: the loop variable and the collection (D-057).
    pub each: Option<(Name, Name)>,
    pub name: Name,
    pub trigger: TriggerExpr,
    pub enable: Option<Expr>,
    pub handler: Vec<OpStmt>,
    pub zeno: Option<ZenoClause>,
    /// `algorithm name on trigger { statements }` (D-101): the event runs these in order.
    pub algorithm: Option<Vec<AlgoStmt>>,
    pub notes: Vec<String>,
    pub span: Span,
}

/// A statement of an algorithm (D-101).
#[derive(Clone, Debug, PartialEq)]
pub enum AlgoStmt {
    Op(OpStmt),
    If { cond: Expr, then: Vec<AlgoStmt>, otherwise: Vec<AlgoStmt>, span: Span },
    While { cond: Expr, body: Vec<AlgoStmt>, span: Span },
    /// `for i in [a, b] { ... }`, `i` a discrete binding of the model.
    For { var: Name, from: Expr, to: Expr, body: Vec<AlgoStmt>, span: Span },
    Step(Span),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TriggerExpr {
    Rising(Expr),
    Falling(Expr),
    Crossing(Expr),
    At(Expr),
    Every(Expr, Option<Expr>),
    Start,
    Input(Name),
    Request(Vec<PayloadDecl>),
    /// `random(λ)`: at random instants, at rate `λ` (D-083).
    Random(Expr),
    /// `on E` or `on E(p: T)`, receiving `E`'s payload as `p` (D-050).
    On(Name, Vec<PayloadDecl>),
}

/// A payload an occurrence receives: `p: T` (D-050), or a member `b in balls` (D-059).
#[derive(Clone, Debug, PartialEq)]
pub struct PayloadDecl {
    pub name: Name,
    pub ty: PayloadTy,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PayloadTy {
    Value(TypeExpr),
    Member(Name),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZenoClause {
    Stop,
    Settle(Vec<OpStmt>),
}

/// A binding, or one component of it (`set vel.y = 0 m/s`), possibly of a member
/// (`set b.vel = ...`, `set row[2].vel.y = ...`, D-057). `b.vel` is read as a binding and a
/// component by the parser; lowering decides which it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub member: Option<Expr>,
    pub name: Name,
    pub component: Option<Name>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpStmt {
    /// `set x = e`; `set T(p) = e` sets a field in each cell (D-091).
    Set { target: Path, coord: Option<Name>, value: Expr, span: Span },
    Contribute { target: Path, value: Expr, span: Span },
    Emit { event: Name, payload: Option<Expr>, span: Span },
    /// `create drops { pos = p }` (D-057).
    Create { part: Name, overrides: Vec<(Name, Expr)>, span: Span },
    /// `destroy b` (D-057).
    Destroy { member: Expr, span: Span },
    /// `connect springs(x, y) { k = e }` (D-058).
    Connect { part: Name, ends: Vec<Expr>, overrides: Vec<(Name, Expr)>, span: Span },
    /// `disconnect s` (D-058).
    Disconnect { relation: Expr, span: Span },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EquationDecl {
    pub name: Name,
    pub lhs: Expr,
    pub rhs: Expr,
    pub checked: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintDecl {
    pub name: Option<Name>,
    pub cond: Expr,
    pub within: Option<Expr>,
    pub policy: Option<Name>,
    /// `with N`: the unknown that holds an enforced inequality (D-098).
    pub force: Option<Name>,
    /// `where |F| <= B`: the bound of the holding force (D-102).
    pub bound: Option<Expr>,
    /// `restitution e` (D-102).
    pub restitution: Option<Expr>,
    pub span: Span,
}

// ---------------------------------------------------------------- types, units, expressions

#[derive(Clone, Debug, PartialEq)]
pub enum TypeExpr {
    /// `Real`, `Velocity`, `Point`, `Quantity<1/L>`, `Vector<Plane, L/T>`.
    Named { name: Name, args: Vec<DimExpr>, span: Span },
    Tuple { items: Vec<TypeExpr>, span: Span },
}

/// A dimension expression inside `<...>`: `M L^2 T^-2`, `1/L`, `Velocity`, or a space name.
#[derive(Clone, Debug, PartialEq)]
pub struct DimExpr {
    pub factors: Vec<DimFactor>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimFactor {
    pub name: Name,
    /// Exponent as a reduced fraction, including the sign from a preceding `/`.
    pub num: i32,
    pub den: i32,
}

/// A unit as written, without spaces: `m/s^2`, `/m`, `N/m`, `deg`, `px`.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitExpr {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Bound {
    Inf,
    Value(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
    pub lo: Bound,
    pub lo_closed: bool,
    pub hi: Bound,
    pub hi_closed: bool,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    pub name: Option<Name>,
    pub value: Expr,
    /// `pos every 0.02 s`: a representation source sampled over the run (PK-6.3).
    pub every: Option<Expr>,
    /// `n at t0 + 7 s over 200 runs`: an ensemble source (D-084), the instant and the count.
    pub ensemble: Option<(Expr, Expr)>,
    /// `gap on decay`: the values at each occurrence of an event (D-089).
    pub on: Option<Name>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Num(f64, Option<UnitExpr>),
    Bool(bool),
    Str(String),
    Name(String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Norm(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `a < b <= c`: one or more comparisons, chained.
    Compare(Box<Expr>, Vec<(CmpOp, Expr)>),
    In(Box<Expr>, Interval),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Otherwise(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Arg>),
    Field(Box<Expr>, Name),
    Index(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    /// `1 m -> 40 px` (presentation scales).
    Map(Box<Expr>, Box<Expr>),
    /// `(pos on landed)` and `(vel on landed microstep 0)` (expectations, PK-3.4).
    On(Box<Expr>, Name, Option<u32>),
    /// `sum(b.m for b in row if b.on)`: an aggregate over a collection (D-055).
    /// `first(e for b in row if c by k)`: the value at the member with the smallest key
    /// (D-106).
    Aggregate { agg: Name, body: Box<Expr>, var: Name, over: Name, filter: Option<Box<Expr>>, by: Option<Box<Expr>> },
    /// `start of b3`, `end of b8` (timeline expectations).
    BeatTime { start: bool, beat: Name },
    /// `match e { case => value, ... }` (MK-10.2, D-049).
    Match(Box<Expr>, Vec<(Name, Expr)>),
}

// ---------------------------------------------------------------- presentations

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationDecl {
    pub name: Name,
    pub model: Name,
    pub items: Vec<PresItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PresItem {
    Observe(Vec<Observation>),
    View(ViewDecl),
    Permit { who: Name, items: Vec<Name> },
    Timeline(Vec<SceneDecl>),
    /// `layout row(scene, column(energy, controls))` (PK-7.4a, D-063).
    Layout(LayoutNode),
    /// `title "..."` (D-076).
    Title(String),
    /// `until t0 + 20 s`: how long the presentation's run lasts (D-091).
    Until(Expr),
    /// `instrument ruler(resolution: 1 mm, range: [0 m, 1 m], noise: 2 mm)` (PK-3.7, D-087).
    Instrument { name: Name, args: Vec<Arg>, span: Span },
}

/// A view, or views in a row or a column; the name of a row or column is its keyword.
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutNode {
    View(Name),
    Row(Name, Vec<LayoutNode>),
    Column(Name, Vec<LayoutNode>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub name: Name,
    pub expr: Expr,
    /// `event_log of bounce`, `diagnostics of conservation`.
    pub of: Option<Name>,
    /// `where zeno_applied`.
    pub filter: Option<Expr>,
    pub schedule: Option<Schedule>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Schedule {
    Live,
    Every(Expr),
    /// `at τ`, or `at τ over n runs` (D-084).
    At(Expr, Option<Expr>),
    On(Name, Option<u32>),
    Over(Interval),
}

/// `view name: kind(args) { reps }`, or `panel name { reps }` (kind `panel`, no arguments).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewDecl {
    pub name: Name,
    pub kind: Name,
    pub args: Vec<Arg>,
    pub reps: Vec<Rep>,
    /// `on click as p request E(p)`: a click on an empty point of the view (D-060).
    pub clicks: Vec<Interaction>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rep {
    /// `for b in row { ... }`: repeated for each member (D-055).
    pub each: Option<(Name, Name)>,
    pub kind: Name,
    pub args: Vec<Arg>,
    pub alias: Option<Name>,
    pub interactions: Vec<Interaction>,
    /// The members of a `group` (D-043).
    pub members: Vec<Rep>,
    pub span: Span,
}

/// `on drag [part] as p { propose x = e }` (PK-5.6).
#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
    pub gesture: Name,
    pub part: Option<Name>,
    /// `on drag live as p`: drag mode `live` (PK-10.9, D-071).
    pub live: bool,
    /// `on drag across n as p`: the normal of the plane a drag moves in (D-104).
    pub across: Option<Expr>,
    pub bind: Name,
    /// `propose x = e`, or a member's binding `propose b.pos = e` (D-059).
    pub proposals: Vec<(Path, Expr)>,
    /// `on click request E(v)`: the event a click requests and its payload (D-059).
    pub request: Option<(Name, Option<Expr>)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneDecl {
    pub name: Name,
    pub beats: Vec<BeatDecl>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeatDecl {
    pub name: Name,
    pub actions: Vec<Action>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    In { view: Name, reps: Vec<Rep> },
    Narrate { text: String, duration: Option<Expr> },
    /// `run rate r [until E]`; with `until` it is two actions, `run rate r` and `wait until E`.
    Run { rate: Expr, until: Option<Name> },
    Hold,
    Highlight(Name),
    /// `hide R [for d] [ease e]`
    Hide(Name, Option<Expr>, Option<Name>),
    /// `reveal fade|draw [for d] [in view] { reps }`
    Reveal { style: Name, duration: Option<Expr>, view: Option<Name>, ease: Option<Name>, reps: Vec<Rep> },
    /// `camera view [to P] [zoom z] [for d]`
    /// `camera v [to P] [zoom z] [azimuth a] [elevation e] [for d] [ease w]` (D-042, D-104).
    Camera { view: Name, center: Option<Expr>, zoom: Option<Expr>, azimuth: Option<Expr>, elevation: Option<Expr>, duration: Option<Expr>, ease: Option<Name> },
    Seek(Expr),
    Show(Rep),
    Explore { limit: Option<Expr>, keep: Vec<Name>, reps: Vec<Rep>, fallback: Vec<Action> },
    Sequence(Vec<Action>),
    Intervene(Vec<OpStmt>),
    Wait(Expr),
    WaitUntil(Name),
    /// `request E` or `request E(value)` (D-050).
    Request(Name, Option<Expr>),
    Reset,
    /// `restart`: the run again from `t0` with the parameters as they are (D-085).
    Restart,
    Branch,
    /// `wait learner [limit d] [fallback { ... }]` (D-067)
    WaitLearner { limit: Option<Expr>, fallback: Vec<Action> },
    /// `animate R opacity|offset to v [for d]` (D-068)
    Animate { target: Name, property: Name, to: Expr, duration: Option<Expr>, ease: Option<Name> },
    /// `release R` (D-068)
    Release(Name),
    /// `bind R [for d]` (D-068)
    Bind(Name, Option<Expr>, Option<Name>),
}

// ---------------------------------------------------------------- runs

#[derive(Clone, Debug, PartialEq)]
pub struct RunDecl {
    pub name: Name,
    pub model: Name,
    pub presentation: Option<Name>,
    pub params: Vec<(Name, Expr)>,
    /// `input { x = v }`, `input { x = v at τ }` (D-051).
    pub inputs: Vec<(Name, Expr, Option<Expr>)>,
    pub config: Vec<(Name, Expr)>,
    pub until: Option<Expr>,
    pub learner: Vec<LearnerStep>,
    pub expects: Vec<Expect>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearnerStep {
    pub at: Expr,
    pub action: LearnerAction,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LearnerAction {
    Continue,
    /// `set slider angle = 60 deg`: the control words, then the value.
    Set { control: Vec<Name>, value: Expr },
    /// `press E`: the button that requests event `E` (D-069).
    Press(Name),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tolerance {
    None,
    Abs(Expr),
    Rel(Expr),
    Exact,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expect {
    Compare { lhs: Expr, rhs: Expr, tol: Tolerance, span: Span },
    In { expr: Expr, interval: Interval, span: Span },
    /// `initialization fails`, `configuration rejected`.
    Outcome { words: Vec<Name>, span: Span },
}
