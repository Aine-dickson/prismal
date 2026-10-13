//! Recursive-descent parser for the working syntax (D-028, D-035).
//!
//! Statements end at a newline or `;` (the lexer drops newlines that continue a statement).
//! A statement that fails to parse is reported and skipped up to the next statement end at
//! the same brace depth, so one error does not hide the rest of the file.

use crate::ast::*;
use crate::lexer::{Comment, Tok, Token};
use crate::{Diag, Span};

/// Reserved words: the words of the model language, expressions and top-level items
/// (working syntax section 1.5, D-040). They are never names.
pub const RESERVED: &[&str] = &[
    "space", "model", "presentation", "run", "object", "const", "param", "input", "state", "discrete", "derived",
    "fn", "flow", "process", "event", "equation", "constraint", "on", "if", "then", "else", "and", "or", "not",
    "otherwise", "in", "where", "true", "false", "zeno", "stop", "settle", "set", "contribute", "create", "destroy",
    "connect", "disconnect", "emit", "enter", "checked", "within", "policy", "reject", "report", "intervenable",
    "private", "symbol", "unit", "rising", "falling", "crossing", "at", "every", "from", "start", "request", "enum",
    "match",
];

/// Contextual keywords: words of presentations, timelines and runs, recognized only where
/// such a word is expected (D-040). Elsewhere they are ordinary names (`process drag`).
pub const CONTEXTUAL: &[&str] = &[
    "for", "view", "panel", "observe", "live", "over", "microstep", "show", "as", "drag", "click", "propose", "permit",
    "timeline", "layout", "row", "column", "scene", "beat", "sequence", "rate", "until", "hold", "seek", "reset", "branch", "intervene", "wait",
    "explore", "limit", "keep", "fallback", "narrate", "highlight", "hide", "reveal", "zoom", "animate", "camera", "bind", "release", "config",
    "expect", "exactly", "rel", "of", "with", "learner", "continue", "press", "undo", "redo", "combine", "ease", "title", "random", "runs", "reseed", "restart",
    "instrument",
];

/// Operators that join two operands; a line that starts with one is reported with a hint
/// (D-081).
const BINARY: &[&str] = &["+", "-", "*", "/", "^", "==", "!=", "<", "<=", ">", ">=", "->", "=>", "."];

pub fn is_reserved(w: &str) -> bool {
    RESERVED.contains(&w)
}

/// Unit symbols recognized after a number: the kernel's units and the presentation unit `px`.
pub fn is_unit_symbol(s: &str) -> bool {
    s == "px" || prismal_ir::Unit::is_symbol(s)
}

type P<T> = Result<T, Diag>;

pub struct Parser<'a> {
    toks: &'a [Token],
    comments: &'a [Comment],
    pos: usize,
    pub diags: Vec<Diag>,
}

impl<'a> Parser<'a> {
    pub fn new(toks: &'a [Token], comments: &'a [Comment]) -> Parser<'a> {
        Parser { toks, comments, pos: 0, diags: vec![] }
    }

    /// One expression and nothing after it.
    pub fn whole_expr(&mut self) -> P<Expr> {
        let e = self.expr()?;
        while matches!(self.peek(), Tok::Newline) {
            self.bump();
        }
        if !matches!(self.peek(), Tok::Eof) {
            return Err(self.unexpected("the end of the value"));
        }
        Ok(e)
    }

    // ------------------------------------------------------------ token helpers

    fn tok(&self) -> &'a Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn peek(&self) -> &'a Tok {
        &self.tok().tok
    }
    fn peek_at(&self, n: usize) -> &'a Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }
    fn span(&self) -> Span {
        self.tok().span
    }
    fn bump(&mut self) -> &'a Token {
        let t = self.tok();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    /// The span from `start` to the end of the last consumed token.
    fn since(&self, start: Span) -> Span {
        let end = if self.pos > 0 { self.toks[self.pos - 1].span.end } else { start.end };
        Span { end: end.max(start.start), ..start }
    }
    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == w)
    }
    fn is_word_at(&self, n: usize, w: &str) -> bool {
        matches!(self.peek_at(n), Tok::Ident(s) if s == w)
    }
    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect_word(&mut self, w: &str) -> P<Span> {
        if self.is_word(w) {
            Ok(self.bump().span)
        } else {
            Err(self.unexpected(&format!("`{w}`")))
        }
    }
    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::Punct(q) if *q == p)
    }
    fn is_punct_at(&self, n: usize, p: &str) -> bool {
        matches!(self.peek_at(n), Tok::Punct(q) if *q == p)
    }
    fn eat_punct(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect_punct(&mut self, p: &str) -> P<Span> {
        if self.is_punct(p) {
            Ok(self.bump().span)
        } else {
            Err(self.unexpected(&format!("`{p}`")))
        }
    }
    fn at_sep(&self) -> bool {
        matches!(self.peek(), Tok::Newline | Tok::Eof) || self.is_punct(";")
    }
    fn at_stmt_end(&self) -> bool {
        self.at_sep() || self.is_punct("}")
    }

    fn describe(t: &Tok) -> String {
        match t {
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Num(n) => format!("number `{n}`"),
            Tok::Str(_) => "a string".into(),
            Tok::Punct(p) => format!("`{p}`"),
            Tok::Newline => "end of line".into(),
            Tok::Eof => "end of input".into(),
        }
    }
    fn unexpected(&self, wanted: &str) -> Diag {
        // A line that starts with an operator: the line before it was a whole statement
        // (D-081). The operator belongs at the end of the line before.
        let line_start = self.pos > 0 && matches!(self.toks[self.pos - 1].tok, Tok::Newline);
        let operator = match self.peek() {
            Tok::Punct(p) => BINARY.contains(p),
            Tok::Ident(w) => w == "and" || w == "or",
            _ => false,
        };
        let hint = match self.peek() {
            _ if line_start && operator => format!(
                ". A line cannot start with {op}: to continue an expression on the next line, end the line before with the operator (`a {op_word}` then `b` on the next line)",
                op = Self::describe(self.peek()),
                op_word = match self.peek() {
                    Tok::Punct(p) => p.to_string(),
                    Tok::Ident(w) => w.clone(),
                    _ => String::new(),
                }
            ),
            Tok::Ident(w) => belongs(w).map(|b| format!(". {b}")).unwrap_or_default(),
            _ => String::new(),
        };
        Diag::new("SX-E02", format!("expected {wanted}, found {}{hint}", Self::describe(self.peek())), self.span())
    }

    /// A clause word after a statement's head (`for d`, `zoom z`, `policy p`): clauses come in
    /// any order, each at most once (D-081). Returns the word read, or `None` at the end.
    fn clause(&mut self, words: &[&'static str], seen: &mut Vec<&'static str>) -> P<Option<&'static str>> {
        for w in words {
            if self.is_word(w) {
                if seen.contains(w) {
                    return Err(Diag::new("SX-E09", format!("`{w}` is given twice"), self.span()));
                }
                seen.push(w);
                self.bump();
                return Ok(Some(w));
            }
        }
        Ok(None)
    }

    /// A declared name: an identifier that is not a reserved word.
    fn name(&mut self, what: &str) -> P<Name> {
        // `der(x) = ...` among declarations: a flow written in the wrong block.
        if matches!(self.peek(), Tok::Ident(s) if s == "der") && matches!(self.peek_at(1), Tok::Punct("(")) {
            return Err(Diag::new("SX-E02", format!("expected {what}, found `der(...)`. {}", belongs("der").unwrap_or_default()), self.span()));
        }
        match self.peek() {
            Tok::Ident(s) if s == "%" => Err(Diag::new("SX-E02", format!("expected {what}, found `%`, the percent unit"), self.span())),
            Tok::Ident(s) if is_reserved(s) => Err(Diag::new(
                "SX-E07",
                format!("`{s}` is a reserved word and cannot be used as {what}"),
                self.span(),
            )),
            Tok::Ident(s) => {
                let span = self.bump().span;
                Ok(Name { text: s.clone(), span })
            }
            _ => Err(self.unexpected(what)),
        }
    }
    /// Any identifier, reserved or not (observation names, argument names, words in runs).
    fn any_name(&mut self, what: &str) -> P<Name> {
        match self.peek() {
            Tok::Ident(s) => {
                let span = self.bump().span;
                Ok(Name { text: s.clone(), span })
            }
            _ => Err(self.unexpected(what)),
        }
    }

    fn skip_seps(&mut self) {
        while matches!(self.peek(), Tok::Newline) || self.is_punct(";") {
            self.bump();
        }
    }

    /// Skips to the end of the current statement at this brace depth.
    fn recover(&mut self) {
        let mut depth = 0;
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::Newline | Tok::Punct(";") if depth == 0 => {
                    self.bump();
                    return;
                }
                Tok::Punct("}") if depth == 0 => return,
                Tok::Punct("{") => depth += 1,
                Tok::Punct("}") => depth -= 1,
                _ => {}
            }
            self.bump();
        }
    }

    /// `{ item (sep item)* }`, recovering from errors in single items.
    fn block<T>(&mut self, mut item: impl FnMut(&mut Self) -> P<Vec<T>>) -> P<Vec<T>> {
        self.expect_punct("{")?;
        let mut out = vec![];
        loop {
            self.skip_seps();
            if self.is_punct("}") {
                self.bump();
                return Ok(out);
            }
            if matches!(self.peek(), Tok::Eof) {
                return Err(Diag::new("SX-E02", "unclosed `{`", self.span()));
            }
            match item(self) {
                Ok(v) => {
                    out.extend(v);
                    if !self.at_stmt_end() {
                        let d = self.unexpected("end of statement");
                        self.diags.push(d);
                        self.recover();
                    }
                }
                Err(d) => {
                    self.diags.push(d);
                    self.recover();
                }
            }
        }
    }

    fn one<T>(x: P<T>) -> P<Vec<T>> {
        x.map(|v| vec![v])
    }

    /// Author notes (D-036): the comment lines directly above `line`, with no blank line between.
    fn notes_before(&self, line: u32) -> Vec<String> {
        let mut lines = vec![];
        let mut want = line;
        for c in self.comments.iter().rev().filter(|c| c.line < line) {
            if !c.own_line || c.line + 1 != want {
                break;
            }
            lines.push(c.text.clone());
            want = c.line;
        }
        if lines.is_empty() {
            return vec![];
        }
        lines.reverse();
        vec![lines.join("\n")]
    }

    // ------------------------------------------------------------ file

    pub fn file(&mut self) -> File {
        let mut items = vec![];
        loop {
            self.skip_seps();
            if matches!(self.peek(), Tok::Eof) {
                break;
            }
            match self.item() {
                Ok(it) => {
                    items.push(it);
                    if !self.at_sep() {
                        let d = self.unexpected("end of line");
                        self.diags.push(d);
                        self.recover();
                    }
                }
                Err(d) => {
                    self.diags.push(d);
                    self.recover();
                    if self.is_punct("}") {
                        self.bump();
                    }
                }
            }
        }
        File { items }
    }

    fn item(&mut self) -> P<Item> {
        let start = self.span();
        let notes = self.notes_before(start.line);
        if self.eat_word("space") {
            let name = self.name("a space name")?;
            self.expect_punct("=")?;
            let kind = self.name("a space kind")?;
            let args = self.args()?;
            return Ok(Item::Space(SpaceDecl { name, kind, args, notes, span: self.since(start) }));
        }
        if self.eat_word("model") {
            let name = self.name("a model name")?;
            let space = if self.eat_word("in") { Some(self.name("a space name")?) } else { None };
            let members = self.block(|p| p.member())?;
            return Ok(Item::Model(ModelDecl { name, space, members, notes, span: self.since(start) }));
        }
        if self.eat_word("presentation") {
            let name = self.name("a presentation name")?;
            self.expect_word("for")?;
            let model = self.name("a model name")?;
            let items = self.block(|p| p.pres_item())?;
            return Ok(Item::Presentation(PresentationDecl { name, model, items, span: self.since(start) }));
        }
        if self.eat_word("run") {
            return self.run_decl(start).map(Item::Run);
        }
        Err(self.unexpected("`space`, `model`, `presentation` or `run`"))
    }

    // ------------------------------------------------------------ model members

    fn member(&mut self) -> P<Vec<Member>> {
        let role = match self.peek() {
            Tok::Ident(w) => match w.as_str() {
                "const" => Some(RoleWord::Const),
                "param" => Some(RoleWord::Param),
                "input" => Some(RoleWord::Input),
                "state" => Some(RoleWord::State),
                "discrete" => Some(RoleWord::Discrete),
                "derived" => Some(RoleWord::Derived),
                _ => None,
            },
            _ => None,
        };
        if let Some(role) = role {
            self.bump();
            return if self.is_punct("{") {
                self.block(|p| Self::one(p.decl(role)))
            } else {
                Ok(vec![self.decl(role)?])
            }
            .map(|v| v.into_iter().map(Member::Decl).collect());
        }
        let start = self.span();
        if self.eat_word("flow") {
            let flows = if self.is_punct("{") { self.block(|p| p.flow_item())? } else { vec![self.flow_stmt()?] };
            return Ok(flows.into_iter().map(Member::Flow).collect());
        }
        // `parts { name: Type [n] { overrides } ... }` (D-055); `parts` is a word only here.
        if self.is_word("parts") && self.is_punct_at(1, "{") {
            self.bump();
            let parts = self.block(|p| Self::one(p.part_decl()))?;
            return Ok(vec![Member::Parts(parts)]);
        }
        if self.is_word("event") {
            return Ok(vec![Member::Event(self.event_decl()?)]);
        }
        // `grid rod(x: [0 m, 1 m], cells: 100)` (D-091); contextual, before a name.
        if self.is_word("grid") && matches!(self.peek_at(1), Tok::Ident(_)) {
            self.bump();
            let name = self.name("a grid name")?;
            let args = self.args()?;
            return Ok(vec![Member::Grid { name, args, span: self.since(start) }]);
        }
        // `boundary T { left = e; right = e }`, `boundary T periodic` (D-091).
        if self.is_word("boundary") && matches!(self.peek_at(1), Tok::Ident(_)) {
            self.bump();
            let field = self.name("a field name")?;
            if self.eat_word("periodic") {
                return Ok(vec![Member::Boundary { field, periodic: true, fixed: vec![], span: self.since(start) }]);
            }
            let fixed = self.block(|p| {
                let edge = p.name("an edge (`left`, `right`)")?;
                p.expect_punct("=")?;
                Ok(vec![(edge, p.expr()?)])
            })?;
            return Ok(vec![Member::Boundary { field, periodic: false, fixed, span: self.since(start) }]);
        }
        // `for b in drops { event ... }`: events repeated per member (D-057).
        if self.eat_word("for") {
            let var = self.name("a member name")?;
            self.expect_word("in")?;
            let over = self.part_path()?;
            let mut events = self.block(|p| {
                if p.is_word("event") {
                    Self::one(p.event_decl())
                } else {
                    Err(p.unexpected("`event` in a `for` block of a model"))
                }
            })?;
            for e in &mut events {
                e.each = Some((var.clone(), over.clone()));
            }
            return Ok(events.into_iter().map(Member::Event).collect());
        }
        if self.eat_word("process") {
            let notes = self.notes_before(start.line);
            let name = self.name("a process name")?;
            let mut flows = vec![];
            let mut events = vec![];
            let items = self.block(|p| {
                if p.eat_word("flow") {
                    let f = if p.is_punct("{") { p.block(|p| p.flow_item())? } else { vec![p.flow_stmt()?] };
                    Ok(f.into_iter().map(Member::Flow).collect())
                } else if p.is_word("event") {
                    Ok(vec![Member::Event(p.event_decl()?)])
                } else {
                    Err(p.unexpected("`flow` or `event` in a process"))
                }
            })?;
            for m in items {
                match m {
                    Member::Flow(f) => flows.push(f),
                    Member::Event(e) => events.push(e),
                    _ => unreachable!(),
                }
            }
            return Ok(vec![Member::Process(ProcessDecl { name, flows, events, notes, span: self.since(start) })]);
        }
        if self.eat_word("fn") {
            let notes = self.notes_before(start.line);
            let name = self.name("a function name")?;
            let params = self.typed_params()?;
            self.expect_punct(":")?;
            let result = self.type_expr()?;
            self.expect_punct("=")?;
            let body = self.expr()?;
            return Ok(vec![Member::Fn(FnDecl { name, params, result, body, notes, span: self.since(start) })]);
        }
        if self.eat_word("enum") {
            // `enum Phase { rising, falling, resting }`, cases separated by commas or lines.
            let notes = self.notes_before(start.line);
            let name = self.name("an enumeration name")?;
            self.expect_punct("{")?;
            let mut cases = vec![];
            loop {
                self.skip_seps();
                if self.eat_punct("}") {
                    break;
                }
                cases.push(self.name("a case name")?);
                if !self.eat_punct(",") && !matches!(self.peek(), Tok::Newline) && !self.is_punct("}") {
                    return Err(self.unexpected("`,` or `}` after a case"));
                }
            }
            return Ok(vec![Member::Enum(EnumDecl { name, cases, notes, span: self.since(start) })]);
        }
        if self.eat_word("equation") {
            let name = self.name("an equation name")?;
            self.expect_punct(":")?;
            let e = self.expr()?;
            let (lhs, rhs) = match e.kind {
                ExprKind::Compare(l, mut rest) if rest.len() == 1 && rest[0].0 == CmpOp::Eq => (*l, rest.remove(0).1),
                _ => return Err(Diag::new("SX-E02", "an equation is written `lhs == rhs`", e.span)),
            };
            let checked = if self.eat_word("checked") {
                self.expect_word("within")?;
                Some(self.expr()?)
            } else {
                None
            };
            return Ok(vec![Member::Equation(EquationDecl { name, lhs, rhs, checked, span: self.since(start) })]);
        }
        if self.eat_word("constraint") {
            let name = if matches!(self.peek(), Tok::Ident(_)) && self.is_punct_at(1, ":") {
                let n = self.name("a constraint name")?;
                self.bump();
                Some(n)
            } else {
                None
            };
            let cond = self.expr()?;
            let (mut within, mut policy, mut seen) = (None, None, vec![]);
            while let Some(w) = self.clause(&["within", "policy"], &mut seen)? {
                if w == "within" {
                    within = Some(self.expr()?);
                } else {
                    policy = Some(self.any_name("a policy")?);
                }
            }
            return Ok(vec![Member::Constraint(ConstraintDecl { name, cond, within, policy, span: self.since(start) })]);
        }
        if self.eat_word("object") {
            let notes = self.notes_before(start.line);
            let name = self.name("an object name")?;
            let members = self.block(|p| p.member())?;
            return Ok(vec![Member::Object(ObjectDecl { name, ends: vec![], undirected: false, members, notes, span: self.since(start) })]);
        }
        // `relation Spring(a in balls, b in balls) { ... }` (D-058), `undirected relation ...`
        // (D-064); `relation` and `undirected` are words only here.
        let undirected = self.is_word("undirected") && self.is_word_at(1, "relation");
        if undirected {
            self.bump();
        }
        if self.is_word("relation") && matches!(self.peek_at(1), Tok::Ident(_)) && self.is_punct_at(2, "(") {
            self.bump();
            let notes = self.notes_before(start.line);
            let name = self.name("a relation name")?;
            self.expect_punct("(")?;
            let mut ends = vec![];
            loop {
                let n = self.name("an endpoint name")?;
                self.expect_word("in")?;
                let mut path = vec![self.name("a collection")?];
                while self.eat_punct(".") {
                    path.push(self.name("a collection")?);
                }
                ends.push((n, path));
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct(")")?;
            let members = if self.is_punct("{") { self.block(|p| p.member())? } else { vec![] };
            return Ok(vec![Member::Object(ObjectDecl { name, ends, undirected, members, notes, span: self.since(start) })]);
        }
        Err(self.unexpected("a declaration (`param`, `state`, `flow`, `event`, ...)"))
    }

    fn typed_params(&mut self) -> P<Vec<(Name, TypeExpr)>> {
        self.expect_punct("(")?;
        let mut out = vec![];
        if !self.is_punct(")") {
            loop {
                let n = self.name("a parameter name")?;
                self.expect_punct(":")?;
                out.push((n, self.type_expr()?));
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")")?;
        Ok(out)
    }

    fn decl(&mut self, role: RoleWord) -> P<Decl> {
        let start = self.span();
        let notes = self.notes_before(start.line);
        let name = self.name("a binding name")?;
        // `T(x in rod)`: a field over a grid (D-091).
        let over = if self.is_punct("(") && matches!(self.peek_at(1), Tok::Ident(_)) && matches!(self.peek_at(2), Tok::Ident(w) if w == "in") {
            self.bump();
            let x = self.name("a coordinate name")?;
            self.expect_word("in")?;
            let g = self.name("a grid name")?;
            self.expect_punct(")")?;
            Some((x, g))
        } else {
            None
        };
        let params = if over.is_none() && self.is_punct("(") { Some(self.typed_params()?) } else { None };
        self.expect_punct(":")?;
        let ty = self.type_expr()?;
        let value = if self.eat_punct("=") { Some(self.expr_no_in()?) } else { None };
        // The range and the modifiers are clauses: any order, each once (D-081).
        let (mut range, mut modifiers, mut seen) = (None, vec![], vec![]);
        let words = ["in", "where", "intervenable", "private", "combine", "symbol", "unit"];
        while let Some(w) = self.clause(&words, &mut seen)? {
            match w {
                "in" | "where" if range.is_some() => {
                    return Err(Diag::new("SX-E09", "a declaration has one range: `in [a, b]` or `where cond`", self.toks[self.pos - 1].span));
                }
                "in" => range = Some(Range::In(self.interval()?)),
                "where" => range = Some(Range::Where(self.expr()?)),
                "intervenable" => modifiers.push(Modifier::Intervenable),
                "private" => modifiers.push(Modifier::Private),
                // `combine sum`: how contributions combine (MK-14.9, D-073).
                "combine" => modifiers.push(Modifier::Combine(self.any_name("a combination (`sum`, `max`, ...)")?)),
                "symbol" => match self.peek().clone() {
                    Tok::Str(s) => {
                        self.bump();
                        modifiers.push(Modifier::Symbol(s));
                    }
                    _ => return Err(self.unexpected("a symbol in quotes")),
                },
                _ => match self.unit()? {
                    Some(u) => modifiers.push(Modifier::Unit(u)),
                    None => return Err(self.unexpected("a unit")),
                },
            }
        }
        Ok(Decl { role, name, params, over, ty, value, range, modifiers, notes, span: self.since(start) })
    }

    /// A flow, or `for b in row { flows }` (D-055).
    fn flow_item(&mut self) -> P<Vec<FlowStmt>> {
        if self.eat_word("for") {
            let var = self.name("a member name")?;
            self.expect_word("in")?;
            let over = self.part_path()?;
            let mut flows = self.block(|p| Self::one(p.flow_stmt()))?;
            for f in &mut flows {
                f.each = Some((var.clone(), over.clone()));
            }
            return Ok(flows);
        }
        Ok(vec![self.flow_stmt()?])
    }

    /// A part: `ball: Ball { pos = ... }` or `row: Ball[3] { pos = ... }` (D-055).
    fn part_decl(&mut self) -> P<PartDecl> {
        let start = self.span();
        let notes = self.notes_before(start.line);
        let name = self.name("a part name")?;
        self.expect_punct(":")?;
        let object = self.name("an object type")?;
        // `[3]`, `[max 50]` or `[3, max 50]` (D-057); `max inf` sets no limit (D-066).
        let (mut count, mut capacity, mut unbounded) = (None, None, false);
        if self.eat_punct("[") {
            if !self.is_word("max") {
                count = Some(self.int()? as u32);
                if self.eat_punct(",") && !self.is_word("max") {
                    return Err(self.unexpected("`max` and the most members the collection holds"));
                }
            }
            if self.eat_word("max") {
                if self.eat_word("inf") {
                    unbounded = true;
                } else {
                    capacity = Some(self.int()? as u32);
                }
            }
            self.expect_punct("]")?;
        }
        let overrides = if self.is_punct("{") {
            self.block(|p| {
                let n = p.name("a binding of the object")?;
                p.expect_punct("=")?;
                Ok(vec![(n, p.expr()?)])
            })?
        } else {
            vec![]
        };
        Ok(PartDecl { name, object, count, capacity, unbounded, overrides, notes, span: self.since(start) })
    }

    fn flow_stmt(&mut self) -> P<FlowStmt> {
        let start = self.span();
        self.expect_word("der")?;
        self.expect_punct("(")?;
        // `der(x)`, or a member's binding: `der(b.vel)`, `der(ball.vel)`, `der(row[2].vel)`.
        let first = self.name("a state name")?;
        let mut member = None;
        let mut target = first.clone();
        if self.is_punct("[") {
            self.bump();
            let i = self.expr()?;
            self.expect_punct("]")?;
            self.expect_punct(".")?;
            let base = Expr { kind: ExprKind::Name(first.text.clone()), span: first.span };
            member = Some(Expr { kind: ExprKind::Index(Box::new(base), Box::new(i)), span: self.since(start) });
            target = self.name("a state name")?;
        } else if self.eat_punct(".") {
            member = Some(Expr { kind: ExprKind::Name(first.text.clone()), span: first.span });
            target = self.name("a state name")?;
        }
        // `der(T(x))`: a field's flow with its coordinate (D-091).
        let coord = if member.is_none() && self.eat_punct("(") {
            let x = self.name("a coordinate name")?;
            self.expect_punct(")")?;
            Some(x)
        } else {
            None
        };
        self.expect_punct(")")?;
        let contribute = if self.eat_punct("+=") {
            true
        } else if self.eat_punct("=") {
            false
        } else {
            return Err(self.unexpected("`=` or `+=`"));
        };
        let expr = self.expr()?;
        Ok(FlowStmt { target, member, each: None, coord, contribute, expr, span: self.since(start) })
    }

    fn event_decl(&mut self) -> P<EventDecl> {
        let start = self.span();
        let notes = self.notes_before(start.line);
        self.expect_word("event")?;
        let name = self.name("an event name")?;
        self.expect_word("on")?;
        let trigger = self.trigger()?;
        let enable = if self.eat_word("if") { Some(self.expr()?) } else { None };
        let handler = if self.is_punct("{") { self.block(|p| Self::one(p.op()))? } else { vec![] };
        // A Zeno clause may follow the handler on the next line.
        if matches!(self.peek(), Tok::Newline) && self.is_word_at(1, "zeno") {
            self.bump();
        }
        let zeno = if self.eat_word("zeno") {
            if self.eat_word("stop") {
                Some(ZenoClause::Stop)
            } else if self.eat_word("settle") {
                Some(ZenoClause::Settle(self.block(|p| Self::one(p.op()))?))
            } else {
                return Err(self.unexpected("`stop` or `settle`"));
            }
        } else {
            None
        };
        Ok(EventDecl { each: None, name, trigger, enable, handler, zeno, notes, span: self.since(start) })
    }

    fn trigger(&mut self) -> P<TriggerExpr> {
        let paren = |p: &mut Self| -> P<Expr> {
            p.expect_punct("(")?;
            let e = p.expr()?;
            p.expect_punct(")")?;
            Ok(e)
        };
        if self.eat_word("rising") {
            return Ok(TriggerExpr::Rising(paren(self)?));
        }
        if self.eat_word("falling") {
            return Ok(TriggerExpr::Falling(paren(self)?));
        }
        if self.eat_word("crossing") {
            return Ok(TriggerExpr::Crossing(paren(self)?));
        }
        if self.eat_word("at") {
            return Ok(TriggerExpr::At(self.expr()?));
        }
        if self.eat_word("every") {
            let period = self.expr()?;
            let from = if self.eat_word("from") { Some(self.expr()?) } else { None };
            return Ok(TriggerExpr::Every(period, from));
        }
        if self.eat_word("start") {
            return Ok(TriggerExpr::Start);
        }
        if self.eat_word("input") {
            self.expect_punct("(")?;
            let n = self.name("an input name")?;
            self.expect_punct(")")?;
            return Ok(TriggerExpr::Input(n));
        }
        if self.eat_word("request") {
            return Ok(TriggerExpr::Request(self.payload_decl()?));
        }
        // `random(λ)` (D-083), a contextual word: `random(p: T)` still follows an event named
        // `random` with a payload.
        if self.is_word("random") && matches!(self.peek_at(1), Tok::Punct("(")) {
            let payload = matches!(self.peek_at(2), Tok::Ident(_)) && (matches!(self.peek_at(3), Tok::Punct(":")) || self.is_word_at(3, "in"));
            if !payload {
                self.bump();
                return Ok(TriggerExpr::Random(paren(self)?));
            }
        }
        match self.peek() {
            Tok::Ident(s) if !is_reserved(s) => {
                let e = self.name("an event name")?;
                Ok(TriggerExpr::On(e, self.payload_decl()?))
            }
            _ => Err(self.unexpected("a trigger (`rising(g)`, `falling(g)`, `crossing(g)`, `at`, `every`, `start`, `input(i)`, `request`, `random(rate)` or an event name)")),
        }
    }

    /// `(p: T)` after `request` or an event name: the payload the occurrence receives
    /// (D-050); a member, `(b in balls)`, or several, `(b in balls, j: Momentum)` (D-059).
    fn payload_decl(&mut self) -> P<Vec<PayloadDecl>> {
        let mut out = vec![];
        if !self.eat_punct("(") {
            return Ok(out);
        }
        loop {
            let name = self.name("a payload name")?;
            let ty = if self.eat_word("in") {
                PayloadTy::Member(self.name("a collection")?)
            } else {
                self.expect_punct(":")?;
                PayloadTy::Value(self.type_expr()?)
            };
            out.push(PayloadDecl { name, ty });
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_punct(")")?;
        Ok(out)
    }

    /// `(v)` or `(v, w)` right after an event name in `emit` and `request`: the payload it
    /// supplies, several as one tuple (D-050, D-059).
    fn payload_args(&mut self) -> P<Option<Expr>> {
        if !self.is_punct("(") || self.tok().space_before {
            return Ok(None);
        }
        let start = self.span();
        self.bump();
        let mut items = vec![self.expr()?];
        while self.eat_punct(",") {
            items.push(self.expr()?);
        }
        self.expect_punct(")")?;
        Ok(Some(if items.len() == 1 { items.pop().unwrap() } else { Expr { kind: ExprKind::Tuple(items), span: self.since(start) } }))
    }

    /// `x`, `x.c`, or a member's binding: `b.x`, `b.x.c`, `row[2].x`, `row[2].x.c` (D-057).
    /// A longer chain names a member through endpoints: `s.b.vel`, `s.b.vel.x` (D-059); the
    /// parser keeps the last two names as binding and component, and lowering decides.
    fn path(&mut self) -> P<Path> {
        let start = self.span();
        let first = self.name("a binding name")?;
        let mut base = Expr { kind: ExprKind::Name(first.text.clone()), span: first.span };
        let indexed = self.eat_punct("[");
        if indexed {
            let i = self.expr()?;
            self.expect_punct("]")?;
            self.expect_punct(".")?;
            base = Expr { kind: ExprKind::Index(Box::new(base), Box::new(i)), span: self.since(start) };
        } else if !self.eat_punct(".") {
            return Ok(Path { member: None, name: first, component: None });
        }
        let mut names = vec![self.name("a binding or a component")?];
        while self.eat_punct(".") {
            names.push(self.name("a binding or a component")?);
        }
        if !indexed && names.len() == 1 {
            return Ok(Path { member: None, name: first, component: names.pop() });
        }
        // The member is the base and every name but the binding and its component.
        let keep = if names.len() >= 2 { 2 } else { 1 };
        let rest = names.split_off(names.len() - keep);
        let mut member = base;
        for n in names {
            member = Expr { span: self.since(start), kind: ExprKind::Field(Box::new(member), n) };
        }
        let mut rest = rest.into_iter();
        let name = rest.next().unwrap();
        Ok(Path { member: Some(member), name, component: rest.next() })
    }

    fn op(&mut self) -> P<OpStmt> {
        let start = self.span();
        if self.eat_word("set") {
            let target = self.path()?;
            self.expect_punct("=")?;
            let value = self.expr()?;
            return Ok(OpStmt::Set { target, value, span: self.since(start) });
        }
        if self.eat_word("contribute") {
            let target = self.path()?;
            self.expect_punct("+=")?;
            let value = self.expr()?;
            return Ok(OpStmt::Contribute { target, value, span: self.since(start) });
        }
        if self.eat_word("emit") {
            let event = self.name("an event name")?;
            let payload = self.payload_args()?;
            return Ok(OpStmt::Emit { event, payload, span: self.since(start) });
        }
        // `create drops`, `create drops { pos = p }` (D-057).
        if self.eat_word("create") {
            // `create left.atoms { ... }`: a collection of a contained object (D-074).
            let part = self.part_path()?;
            let overrides = if self.is_punct("{") {
                self.block(|p| {
                    let n = p.name("a binding of the object")?;
                    p.expect_punct("=")?;
                    Ok(vec![(n, p.expr()?)])
                })?
            } else {
                vec![]
            };
            return Ok(OpStmt::Create { part, overrides, span: self.since(start) });
        }
        if self.eat_word("destroy") {
            let member = self.postfix_expr()?;
            return Ok(OpStmt::Destroy { member, span: self.since(start) });
        }
        // `connect springs(x, y)`, `connect springs(x, y) { k = e }` (D-058).
        if self.eat_word("connect") {
            let part = self.part_path()?;
            self.expect_punct("(")?;
            let mut ends = vec![];
            loop {
                ends.push(self.expr()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct(")")?;
            let overrides = if self.is_punct("{") {
                self.block(|p| {
                    let n = p.name("a binding of the relation")?;
                    p.expect_punct("=")?;
                    Ok(vec![(n, p.expr()?)])
                })?
            } else {
                vec![]
            };
            return Ok(OpStmt::Connect { part, ends, overrides, span: self.since(start) });
        }
        if self.eat_word("disconnect") {
            let relation = self.postfix_expr()?;
            return Ok(OpStmt::Disconnect { relation, span: self.since(start) });
        }
        Err(self.unexpected("an operation (`set`, `contribute`, `emit`, `create`, `destroy`, `connect`, `disconnect`)"))
    }

    // ------------------------------------------------------------ types and units

    fn type_expr(&mut self) -> P<TypeExpr> {
        let start = self.span();
        if self.eat_punct("(") {
            let mut items = vec![self.type_expr()?];
            while self.eat_punct(",") {
                items.push(self.type_expr()?);
            }
            self.expect_punct(")")?;
            return Ok(TypeExpr::Tuple { items, span: self.since(start) });
        }
        let name = self.name("a type")?;
        let mut args = vec![];
        if self.eat_punct("<") {
            loop {
                args.push(self.dim_expr()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct(">")?;
        }
        Ok(TypeExpr::Named { name, args, span: self.since(start) })
    }

    /// `M L^2 T^-2`, `1/L`, `L/T^2`, `M/(L T^2)` is not supported: write `M/L/T^2`.
    fn dim_expr(&mut self) -> P<DimExpr> {
        let start = self.span();
        let mut factors = vec![];
        let mut sign = 1;
        loop {
            let name = match self.peek().clone() {
                Tok::Ident(s) => Name { text: s, span: self.bump().span },
                Tok::Num(1.0) => Name { text: "1".into(), span: self.bump().span },
                _ => return Err(self.unexpected("a dimension")),
            };
            let (mut num, mut den) = (1, 1);
            if self.eat_punct("^") {
                let neg = self.eat_punct("-");
                if self.eat_punct("(") {
                    let neg2 = self.eat_punct("-");
                    num = self.int()?;
                    self.expect_punct("/")?;
                    den = self.int()?;
                    self.expect_punct(")")?;
                    if neg2 {
                        num = -num;
                    }
                } else {
                    num = self.int()?;
                }
                if neg {
                    num = -num;
                }
            }
            factors.push(DimFactor { name, num: sign * num, den });
            if self.eat_punct("/") {
                sign = -1;
            } else if self.eat_punct("*") {
            } else if !matches!(self.peek(), Tok::Ident(_)) {
                break;
            }
        }
        Ok(DimExpr { factors, span: self.since(start) })
    }

    fn int(&mut self) -> P<i32> {
        match self.peek() {
            Tok::Num(n) if n.fract() == 0.0 && n.abs() < 1e6 => {
                let v = *n as i32;
                self.bump();
                Ok(v)
            }
            _ => Err(self.unexpected("an integer")),
        }
    }

    /// A unit written without spaces after a number: `m/s^2`, `/m`, `N/m`, `deg`.
    fn unit(&mut self) -> P<Option<UnitExpr>> {
        let start = self.span();
        let mut text = String::new();
        match self.peek() {
            Tok::Ident(s) if is_unit_symbol(s) => {
                text.push_str(s);
                self.bump();
            }
            Tok::Punct("/") => match self.peek_at(1) {
                Tok::Ident(s) if is_unit_symbol(s) && !self.toks[self.pos + 1].space_before => {
                    text.push('/');
                    text.push_str(s);
                    self.bump();
                    self.bump();
                }
                _ => return Ok(None),
            },
            _ => return Ok(None),
        }
        loop {
            if self.tok().space_before {
                break;
            }
            if self.is_punct("^") {
                self.bump();
                let neg = !self.tok().space_before && self.eat_punct("-");
                if self.tok().space_before {
                    return Err(self.unexpected("an exponent written without spaces"));
                }
                // A whole or half number (D-090): `s^2`, `s^0.5`.
                let n = match self.peek() {
                    Tok::Num(n) if (2.0 * n).fract() == 0.0 && n.abs() < 1e6 => *n,
                    _ => return Err(self.unexpected("a whole or half number as a unit's exponent")),
                };
                self.bump();
                text.push_str(&format!("^{}{n}", if neg { "-" } else { "" }));
                continue;
            }
            if self.is_punct("*") || self.is_punct("/") {
                let next = &self.toks[(self.pos + 1).min(self.toks.len() - 1)];
                if let Tok::Ident(s) = &next.tok {
                    if is_unit_symbol(s) && !next.space_before {
                        text.push_str(if self.is_punct("*") { "*" } else { "/" });
                        text.push_str(s);
                        self.bump();
                        self.bump();
                        continue;
                    }
                }
            }
            break;
        }
        Ok(Some(UnitExpr { text, span: self.since(start) }))
    }

    // ------------------------------------------------------------ expressions

    pub fn expr(&mut self) -> P<Expr> {
        self.expr_in(true)
    }
    /// An expression in which `in` is not an operator: a declaration's value, where `in`
    /// starts a range.
    fn expr_no_in(&mut self) -> P<Expr> {
        self.expr_in(false)
    }

    fn expr_in(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        if self.eat_word("if") {
            let c = self.expr()?;
            self.expect_word("then")?;
            let a = self.expr_in(allow_in)?;
            self.expect_word("else")?;
            let b = self.expr_in(allow_in)?;
            return Ok(Expr { kind: ExprKind::If(Box::new(c), Box::new(a), Box::new(b)), span: self.since(start) });
        }
        let mut l = self.map_expr(allow_in)?;
        while self.eat_word("otherwise") {
            let r = self.map_expr(allow_in)?;
            l = Expr { kind: ExprKind::Otherwise(Box::new(l), Box::new(r)), span: self.since(start) };
        }
        Ok(l)
    }

    fn map_expr(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        let l = self.or_expr(allow_in)?;
        if self.eat_punct("->") {
            let r = self.or_expr(allow_in)?;
            return Ok(Expr { kind: ExprKind::Map(Box::new(l), Box::new(r)), span: self.since(start) });
        }
        Ok(l)
    }

    fn or_expr(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        let mut l = self.and_expr(allow_in)?;
        while self.eat_word("or") {
            let r = self.and_expr(allow_in)?;
            l = Expr { kind: ExprKind::Binary(BinOp::Or, Box::new(l), Box::new(r)), span: self.since(start) };
        }
        Ok(l)
    }

    fn and_expr(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        let mut l = self.not_expr(allow_in)?;
        while self.eat_word("and") {
            let r = self.not_expr(allow_in)?;
            l = Expr { kind: ExprKind::Binary(BinOp::And, Box::new(l), Box::new(r)), span: self.since(start) };
        }
        Ok(l)
    }

    fn not_expr(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        if self.eat_word("not") {
            let e = self.not_expr(allow_in)?;
            return Ok(Expr { kind: ExprKind::Not(Box::new(e)), span: self.since(start) });
        }
        self.cmp_expr(allow_in)
    }

    fn cmp_expr(&mut self, allow_in: bool) -> P<Expr> {
        let start = self.span();
        let first = self.add_expr()?;
        let mut rest = vec![];
        loop {
            let op = match self.peek() {
                Tok::Punct("==") => CmpOp::Eq,
                Tok::Punct("!=") => CmpOp::Ne,
                Tok::Punct("<") => CmpOp::Lt,
                Tok::Punct("<=") => CmpOp::Le,
                Tok::Punct(">") => CmpOp::Gt,
                Tok::Punct(">=") => CmpOp::Ge,
                Tok::Punct("=") => {
                    return Err(Diag::new("SX-E02", "`=` gives a value; write `==` for equality (D-035)", self.span()));
                }
                _ => break,
            };
            self.bump();
            rest.push((op, self.add_expr()?));
        }
        let e = if rest.is_empty() {
            first
        } else {
            Expr { kind: ExprKind::Compare(Box::new(first), rest), span: self.since(start) }
        };
        if allow_in && self.is_word("in") && !self.is_punct_at(1, "{") {
            self.bump();
            let i = self.interval()?;
            return Ok(Expr { kind: ExprKind::In(Box::new(e), i), span: self.since(start) });
        }
        Ok(e)
    }

    fn add_expr(&mut self) -> P<Expr> {
        let start = self.span();
        let mut l = self.mul_expr()?;
        loop {
            let op = if self.is_punct("+") {
                BinOp::Add
            } else if self.is_punct("-") {
                BinOp::Sub
            } else {
                break;
            };
            self.bump();
            let r = self.mul_expr()?;
            l = Expr { kind: ExprKind::Binary(op, Box::new(l), Box::new(r)), span: self.since(start) };
        }
        Ok(l)
    }

    fn mul_expr(&mut self) -> P<Expr> {
        let start = self.span();
        let mut l = self.unary_expr()?;
        loop {
            let op = if self.is_punct("*") {
                BinOp::Mul
            } else if self.is_punct("/") {
                BinOp::Div
            } else {
                break;
            };
            self.bump();
            let r = self.unary_expr()?;
            l = Expr { kind: ExprKind::Binary(op, Box::new(l), Box::new(r)), span: self.since(start) };
        }
        Ok(l)
    }

    fn unary_expr(&mut self) -> P<Expr> {
        let start = self.span();
        if self.eat_punct("-") {
            let e = self.unary_expr()?;
            return Ok(Expr { kind: ExprKind::Neg(Box::new(e)), span: self.since(start) });
        }
        self.pow_expr()
    }

    /// `^` binds tighter than unary minus and is right-associative: `-x^2` is `-(x^2)`.
    fn pow_expr(&mut self) -> P<Expr> {
        let start = self.span();
        let base = self.postfix_expr()?;
        if self.eat_punct("^") {
            let e = self.unary_expr()?;
            return Ok(Expr { kind: ExprKind::Binary(BinOp::Pow, Box::new(base), Box::new(e)), span: self.since(start) });
        }
        Ok(base)
    }

    fn postfix_expr(&mut self) -> P<Expr> {
        let start = self.span();
        let mut e = self.primary()?;
        loop {
            if self.is_punct(".") {
                self.bump();
                let n = self.any_name("a component or field name")?;
                e = Expr { kind: ExprKind::Field(Box::new(e), n), span: self.since(start) };
            } else if self.is_punct("(") && !self.tok().space_before {
                let args = self.args()?;
                e = Expr { kind: ExprKind::Call(Box::new(e), args), span: self.since(start) };
            } else if self.is_punct("[") && !self.tok().space_before {
                self.bump();
                let i = self.expr()?;
                self.expect_punct("]")?;
                e = Expr { kind: ExprKind::Index(Box::new(e), Box::new(i)), span: self.since(start) };
            } else {
                return Ok(e);
            }
        }
    }

    /// `(a, name: b, ...)`.
    fn args(&mut self) -> P<Vec<Arg>> {
        self.expect_punct("(")?;
        let mut out = vec![];
        if !self.is_punct(")") {
            loop {
                let name = if matches!(self.peek(), Tok::Ident(_)) && self.is_punct_at(1, ":") {
                    let n = self.any_name("an argument name")?;
                    self.bump();
                    Some(n)
                } else {
                    None
                };
                let mut value = self.expr()?;
                // `sum(e for b in row [if c])`: an aggregate (D-055). `in` here is not the
                // interval test, so the loop is read word by word.
                if self.is_word("for") {
                    let s = value.span;
                    self.bump();
                    let var = self.name("a member name")?;
                    self.expect_word("in")?;
                    let over = self.part_path()?;
                    let filter = if self.eat_word("if") { Some(Box::new(self.expr()?)) } else { None };
                    let agg = Name { text: String::new(), span: s };
                    value = Expr { kind: ExprKind::Aggregate { agg, body: Box::new(value), var, over, filter }, span: self.since(s) };
                }
                let every = if self.eat_word("every") { Some(self.expr()?) } else { None };
                // An ensemble source (D-084): `n at t0 + 7 s over 200 runs`.
                let ensemble = if every.is_none() && self.eat_word("at") {
                    let at = self.expr()?;
                    self.expect_word("over")?;
                    let n = self.expr()?;
                    self.expect_word("runs")?;
                    Some((at, n))
                } else {
                    None
                };
                // The values at each occurrence of an event (D-089): `gap on decay`.
                let on = if every.is_none() && ensemble.is_none() && self.eat_word("on") { Some(self.name("an event name")?) } else { None };
                out.push(Arg { name, value, every, ensemble, on });
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")")?;
        Ok(out)
    }

    fn primary(&mut self) -> P<Expr> {
        let start = self.span();
        let t = self.peek().clone();
        match t {
            Tok::Num(v) => {
                self.bump();
                let unit = self.unit()?;
                let lit = Expr { kind: ExprKind::Num(v, unit), span: self.since(start) };
                // `2π`: a number directly followed by a named constant is a product (section 1.5).
                if let Tok::Ident(s) = self.peek() {
                    if !self.tok().space_before {
                        if s == "π" || s == "pi" {
                            let c = Expr { kind: ExprKind::Name(s.clone()), span: self.bump().span };
                            return Ok(Expr { kind: ExprKind::Binary(BinOp::Mul, Box::new(lit), Box::new(c)), span: self.since(start) });
                        }
                        return Err(Diag::new(
                            "SX-E02",
                            format!("`{s}` directly after a number: write a space before a unit, or `*` for a product"),
                            self.span(),
                        ));
                    }
                }
                Ok(lit)
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr { kind: ExprKind::Str(s), span: start })
            }
            Tok::Ident(w) => match w.as_str() {
                "true" | "false" => {
                    self.bump();
                    Ok(Expr { kind: ExprKind::Bool(w == "true"), span: start })
                }
                "start" | "end" if self.is_word_at(1, "of") => {
                    self.bump();
                    self.bump();
                    let beat = self.name("a beat name")?;
                    Ok(Expr { kind: ExprKind::BeatTime { start: w == "start", beat }, span: self.since(start) })
                }
                "if" => self.expr(),
                "match" => {
                    // `match e { case => value, ... }` (MK-10.2), arms separated by commas or lines.
                    self.bump();
                    let scrutinee = self.expr()?;
                    self.expect_punct("{")?;
                    let mut arms = vec![];
                    loop {
                        self.skip_seps();
                        if self.eat_punct("}") {
                            break;
                        }
                        let case = self.name("a case name")?;
                        self.expect_punct("=>")?;
                        let value = self.expr()?;
                        arms.push((case, value));
                        if !self.eat_punct(",") && !matches!(self.peek(), Tok::Newline) && !self.is_punct("}") {
                            return Err(self.unexpected("`,` or `}` after an arm"));
                        }
                    }
                    Ok(Expr { kind: ExprKind::Match(Box::new(scrutinee), arms), span: self.since(start) })
                }
                _ if is_reserved(&w) => Err(self.unexpected("an expression")),
                _ => {
                    self.bump();
                    Ok(Expr { kind: ExprKind::Name(w), span: start })
                }
            },
            Tok::Punct("(") => {
                self.bump();
                let first = self.expr()?;
                if self.eat_word("on") {
                    let ev = self.name("an event name")?;
                    let micro = if self.eat_word("microstep") { Some(self.int()? as u32) } else { None };
                    self.expect_punct(")")?;
                    return Ok(Expr { kind: ExprKind::On(Box::new(first), ev, micro), span: self.since(start) });
                }
                if self.eat_punct(")") {
                    return Ok(Expr { span: self.since(start), ..first });
                }
                let mut items = vec![first];
                while self.eat_punct(",") {
                    items.push(self.expr()?);
                }
                self.expect_punct(")")?;
                Ok(Expr { kind: ExprKind::Tuple(items), span: self.since(start) })
            }
            Tok::Punct("[") => {
                self.bump();
                let mut items = vec![];
                if !self.is_punct("]") {
                    loop {
                        items.push(self.expr()?);
                        if !self.eat_punct(",") {
                            break;
                        }
                    }
                }
                self.expect_punct("]")?;
                Ok(Expr { kind: ExprKind::List(items), span: self.since(start) })
            }
            Tok::Punct("|") => {
                self.bump();
                let e = self.expr()?;
                self.expect_punct("|")?;
                Ok(Expr { kind: ExprKind::Norm(Box::new(e)), span: self.since(start) })
            }
            _ => Err(self.unexpected("an expression")),
        }
    }

    /// `[a, b]`, `(a, b)`, `[a, b)`, `(a, b]`, with `inf` or `-inf` for an unbounded end.
    fn interval(&mut self) -> P<Interval> {
        let start = self.span();
        let lo_closed = if self.eat_punct("[") {
            true
        } else if self.eat_punct("(") {
            false
        } else {
            return Err(self.unexpected("an interval (`[a, b]`, `(a, b)`, `[a, b)`, `(a, b]`)"));
        };
        let lo = self.bound()?;
        self.expect_punct(",")?;
        let hi = self.bound()?;
        let hi_closed = if self.eat_punct("]") {
            true
        } else if self.eat_punct(")") {
            false
        } else {
            return Err(self.unexpected("`]` or `)`"));
        };
        Ok(Interval { lo, lo_closed, hi, hi_closed, span: self.since(start) })
    }

    fn bound(&mut self) -> P<Bound> {
        let inf_next = |p: &Self, n: usize| matches!(p.peek_at(n), Tok::Ident(s) if s == "inf") && matches!(p.peek_at(n + 1), Tok::Punct("," | "]" | ")"));
        if inf_next(self, 0) {
            self.bump();
            return Ok(Bound::Inf);
        }
        if self.is_punct("-") && inf_next(self, 1) {
            self.bump();
            self.bump();
            return Ok(Bound::Inf);
        }
        Ok(Bound::Value(Box::new(self.expr()?)))
    }

    /// A collection, `balls`, or a path through contained objects, `left.atoms` (D-065), as
    /// one name with dots.
    fn part_path(&mut self) -> P<Name> {
        let mut n = self.name("a collection")?;
        while self.eat_punct(".") {
            let next = self.name("a collection")?;
            n.text = format!("{}.{}", n.text, next.text);
            n.span = Span { end: next.span.end, ..n.span };
        }
        Ok(n)
    }

    // ------------------------------------------------------------ presentations

    fn pres_item(&mut self) -> P<Vec<PresItem>> {
        let start = self.span();
        if self.eat_word("observe") {
            let obs = if self.is_punct("{") { self.block(|p| Self::one(p.observation()))? } else { vec![self.observation()?] };
            return Ok(vec![PresItem::Observe(obs)]);
        }
        if self.eat_word("view") {
            let name = self.name("a view name")?;
            self.expect_punct(":")?;
            let kind = self.name("a view kind")?;
            let args = self.args()?;
            let (reps, clicks) = self.view_body()?;
            return Ok(vec![PresItem::View(ViewDecl { name, kind, args, reps, clicks, span: self.since(start) })]);
        }
        if self.is_word("panel") {
            let kind = self.any_name("`panel`")?;
            let name = self.name("a panel name")?;
            let (reps, clicks) = self.view_body()?;
            return Ok(vec![PresItem::View(ViewDecl { name, kind, args: vec![], reps, clicks, span: self.since(start) })]);
        }
        if self.eat_word("permit") {
            let who = self.name("a role")?;
            let items = self.block(|p| Self::one(p.name("a permission")))?;
            return Ok(vec![PresItem::Permit { who, items }]);
        }
        if self.eat_word("timeline") {
            let scenes = self.block(|p| Self::one(p.scene()))?;
            return Ok(vec![PresItem::Timeline(scenes)]);
        }
        if self.eat_word("layout") {
            return Ok(vec![PresItem::Layout(self.layout_node()?)]);
        }
        // `instrument ruler(resolution: 1 mm, noise: 2 mm)` (PK-3.7, D-087).
        if self.eat_word("instrument") {
            let name = self.name("an instrument name")?;
            let args = self.args()?;
            return Ok(vec![PresItem::Instrument { name, args, span: self.since(start) }]);
        }
        // `title "Projectile motion"` (D-076).
        if self.eat_word("title") {
            return match self.peek().clone() {
                Tok::Str(s) => {
                    self.bump();
                    Ok(vec![PresItem::Title(s)])
                }
                _ => Err(self.unexpected("a title in quotes")),
            };
        }
        Err(self.unexpected("`observe`, `view`, `panel`, `instrument`, `permit`, `timeline` or `layout`"))
    }

    /// A view name, or `row(...)` or `column(...)` of layout nodes.
    fn layout_node(&mut self) -> P<LayoutNode> {
        let name = self.name("a view name, `row(...)` or `column(...)`")?;
        if !matches!(name.text.as_str(), "row" | "column") || !self.is_punct("(") {
            return Ok(LayoutNode::View(name));
        }
        self.expect_punct("(")?;
        let mut items = vec![self.layout_node()?];
        while self.eat_punct(",") && !self.is_punct(")") {
            items.push(self.layout_node()?);
        }
        self.expect_punct(")")?;
        Ok(if name.text == "row" { LayoutNode::Row(name, items) } else { LayoutNode::Column(name, items) })
    }

    fn observation(&mut self) -> P<Observation> {
        let start = self.span();
        let name = self.any_name("an observation name")?;
        self.expect_punct("=")?;
        let expr = self.expr()?;
        let of = if self.eat_word("of") { Some(self.name("an element name")?) } else { None };
        let filter = if self.eat_word("where") { Some(self.expr()?) } else { None };
        let schedule = if self.eat_word("live") {
            Some(Schedule::Live)
        } else if self.eat_word("every") {
            Some(Schedule::Every(self.expr()?))
        } else if self.eat_word("at") {
            let at = self.expr()?;
            // `at τ over n runs` (D-084).
            let runs = if self.eat_word("over") {
                let n = self.expr()?;
                self.expect_word("runs")?;
                Some(n)
            } else {
                None
            };
            Some(Schedule::At(at, runs))
        } else if self.eat_word("on") {
            let ev = self.name("an event name")?;
            let micro = if self.eat_word("microstep") { Some(self.int()? as u32) } else { None };
            Some(Schedule::On(ev, micro))
        } else if self.eat_word("over") {
            Some(Schedule::Over(self.interval()?))
        } else {
            None
        };
        Ok(Observation { name, expr, of, filter, schedule, span: self.since(start) })
    }

    /// A view's block: representations, and `on click as p request E(p)` for a click on a
    /// point of the view that no representation takes (D-060).
    fn view_body(&mut self) -> P<(Vec<Rep>, Vec<Interaction>)> {
        let items = self.block(|p| {
            if p.is_word("on") {
                let start = p.span();
                p.bump();
                let gesture = p.any_name("`click`")?;
                if gesture.text != "click" {
                    return Err(Diag::new("SX-E02", "a view takes `on click as p request E(p)`; drags belong to representations", gesture.span));
                }
                p.expect_word("as")?;
                let bind = p.name("a name for the point clicked")?;
                p.expect_word("request")?;
                let event = p.name("an event name")?;
                let payload = p.payload_args()?;
                return Ok(vec![Err(Interaction { gesture, part: None, live: false, bind, proposals: vec![], request: Some((event, payload)), span: p.since(start) })]);
            }
            Ok(p.rep_item()?.into_iter().map(Ok).collect())
        })?;
        let (mut reps, mut clicks) = (vec![], vec![]);
        for i in items {
            match i {
                Ok(r) => reps.push(r),
                Err(c) => clicks.push(c),
            }
        }
        Ok((reps, clicks))
    }

    /// A representation, or `for b in row { representations }`, each repeated per member
    /// (D-055).
    fn rep_item(&mut self) -> P<Vec<Rep>> {
        if self.eat_word("for") {
            let var = self.name("a member name")?;
            self.expect_word("in")?;
            let over = self.part_path()?;
            let mut reps = self.block(|p| p.rep_item())?;
            for r in &mut reps {
                if r.each.is_some() {
                    return Err(Diag::new("SX-E06", "a `for` inside a `for` is not in v0", r.span));
                }
                r.each = Some((var.clone(), over.clone()));
            }
            return Ok(reps);
        }
        Ok(vec![self.rep()?])
    }

    fn rep(&mut self) -> P<Rep> {
        let start = self.span();
        // `equation` is reserved in models and is also a representation kind (PK-6.3); in a
        // representation's position it is that kind, never a name.
        let kind = if self.is_word("equation") { self.any_name("a representation")? } else { self.name("a representation")? };
        let args = if self.is_punct("(") { self.args()? } else { vec![] };
        let alias = if self.eat_word("as") { Some(self.name("a representation name")?) } else { None };
        // A group's block holds its members (D-043); any other kind's, its interactions.
        let (mut interactions, mut members) = (vec![], vec![]);
        if self.is_punct("{") {
            if kind.text == "group" {
                members = self.block(|p| p.rep_item())?;
            } else {
                interactions = self.block(|p| Self::one(p.interaction()))?;
            }
        }
        Ok(Rep { each: None, kind, args, alias, interactions, members, span: self.since(start) })
    }

    fn interaction(&mut self) -> P<Interaction> {
        let start = self.span();
        self.expect_word("on")?;
        let gesture = self.any_name("a gesture (`drag`, `click`)")?;
        // `on click request E(v)` (D-059).
        if gesture.text == "click" {
            self.expect_word("request")?;
            let event = self.name("an event name")?;
            let payload = self.payload_args()?;
            let bind = gesture.clone();
            return Ok(Interaction { gesture, part: None, live: false, bind, proposals: vec![], request: Some((event, payload)), span: self.since(start) });
        }
        let part = if !self.is_word("as") && !self.is_word("live") { Some(self.name("a part")?) } else { None };
        // `on drag [part] live as p`: commits while the run advances (D-071).
        let live = self.is_word("live");
        if live {
            self.bump();
        }
        self.expect_word("as")?;
        let bind = self.name("a name for the gesture value")?;
        let proposals = self.block(|p| {
            p.expect_word("propose")?;
            let n = p.path()?;
            p.expect_punct("=")?;
            Ok(vec![(n, p.expr()?)])
        })?;
        Ok(Interaction { gesture, part, live, bind, proposals, request: None, span: self.since(start) })
    }

    fn scene(&mut self) -> P<SceneDecl> {
        let start = self.span();
        self.expect_word("scene")?;
        let name = self.name("a scene name")?;
        let beats = self.block(|p| {
            let s = p.span();
            p.expect_word("beat")?;
            let name = p.name("a beat name")?;
            let actions = p.block(|p| Self::one(p.action()))?;
            Ok(vec![BeatDecl { name, actions, span: p.since(s) }])
        })?;
        Ok(SceneDecl { name, beats, span: self.since(start) })
    }

    fn action(&mut self) -> P<Action> {
        if self.eat_word("in") {
            let view = self.name("a view name")?;
            let reps = self.block(|p| p.rep_item())?;
            return Ok(Action::In { view, reps });
        }
        if self.eat_word("narrate") {
            let text = match self.peek().clone() {
                Tok::Str(s) => {
                    self.bump();
                    s
                }
                _ => return Err(self.unexpected("narration text in quotes")),
            };
            let duration = if self.eat_word("for") { Some(self.expr()?) } else { None };
            return Ok(Action::Narrate { text, duration });
        }
        if self.eat_word("run") {
            self.expect_word("rate")?;
            let rate = self.expr()?;
            let until = if self.eat_word("until") { Some(self.name("an event name")?) } else { None };
            return Ok(Action::Run { rate, until });
        }
        if self.eat_word("hold") {
            return Ok(Action::Hold);
        }
        if self.eat_word("reset") {
            return Ok(Action::Reset);
        }
        if self.eat_word("restart") {
            return Ok(Action::Restart);
        }
        if self.eat_word("branch") {
            return Ok(Action::Branch);
        }
        if self.eat_word("highlight") {
            return Ok(Action::Highlight(self.name("a representation name")?));
        }
        if self.eat_word("hide") {
            let n = self.name("a representation name")?;
            let d = if self.eat_word("for") { Some(self.expr()?) } else { None };
            return Ok(Action::Hide(n, d));
        }
        if self.eat_word("reveal") {
            let style = self.any_name("`fade` or `draw`")?;
            let (mut duration, mut view, mut seen) = (None, None, vec![]);
            while let Some(w) = self.clause(&["for", "in"], &mut seen)? {
                if w == "for" {
                    duration = Some(self.expr_no_in()?);
                } else {
                    view = Some(self.name("a view name")?);
                }
            }
            let reps = self.block(|p| p.rep_item())?;
            return Ok(Action::Reveal { style, duration, view, reps });
        }
        if self.eat_word("camera") {
            let view = self.name("a view name")?;
            let (mut center, mut zoom, mut duration, mut seen) = (None, None, None, vec![]);
            while let Some(w) = self.clause(&["to", "zoom", "for"], &mut seen)? {
                let e = Some(self.expr()?);
                match w {
                    "to" => center = e,
                    "zoom" => zoom = e,
                    _ => duration = e,
                }
            }
            return Ok(Action::Camera { view, center, zoom, duration });
        }
        if self.eat_word("seek") {
            return Ok(Action::Seek(self.expr()?));
        }
        if self.eat_word("show") {
            return Ok(Action::Show(self.rep()?));
        }
        if self.eat_word("explore") {
            let (mut limit, mut keep, mut seen) = (None, vec![], vec![]);
            while let Some(w) = self.clause(&["limit", "keep"], &mut seen)? {
                if w == "limit" {
                    limit = Some(self.expr()?);
                } else {
                    keep.push(self.name("a binding name")?);
                    while self.eat_punct(",") {
                        keep.push(self.name("a binding name")?);
                    }
                }
            }
            let reps = self.block(|p| p.rep_item())?;
            let fallback = if self.eat_word("fallback") { self.block(|p| Self::one(p.action()))? } else { vec![] };
            return Ok(Action::Explore { limit, keep, reps, fallback });
        }
        if self.eat_word("sequence") {
            return Ok(Action::Sequence(self.block(|p| Self::one(p.action()))?));
        }
        if self.eat_word("intervene") {
            return Ok(Action::Intervene(self.block(|p| Self::one(p.op()))?));
        }
        if self.eat_word("wait") {
            if self.eat_word("until") {
                return Ok(Action::WaitUntil(self.name("an event name")?));
            }
            if self.eat_word("learner") {
                let limit = if self.eat_word("limit") { Some(self.expr()?) } else { None };
                let fallback = if self.eat_word("fallback") { self.block(|p| Self::one(p.action()))? } else { vec![] };
                return Ok(Action::WaitLearner { limit, fallback });
            }
            return Ok(Action::Wait(self.expr()?));
        }
        if self.eat_word("request") {
            let e = self.name("an event name")?;
            let payload = self.payload_args()?;
            return Ok(Action::Request(e, payload));
        }
        if self.eat_word("animate") {
            let target = self.name("a representation name")?;
            let property = self.any_name("`opacity`, `offset`, `color`, `line`, `scale` or `morph`")?;
            self.expect_word("to")?;
            let to = self.expr()?;
            // `ease linear`: the easing function (D-075).
            let (mut duration, mut ease, mut seen) = (None, None, vec![]);
            while let Some(w) = self.clause(&["for", "ease"], &mut seen)? {
                if w == "for" {
                    duration = Some(self.expr()?);
                } else {
                    ease = Some(self.any_name("an easing function (`linear`, `smooth`, `in`, `out`)")?);
                }
            }
            return Ok(Action::Animate { target, property, to, duration, ease });
        }
        if self.eat_word("release") {
            return Ok(Action::Release(self.name("a representation name")?));
        }
        if self.eat_word("bind") {
            let n = self.name("a representation name")?;
            let d = if self.eat_word("for") { Some(self.expr()?) } else { None };
            return Ok(Action::Bind(n, d));
        }
        Err(self.unexpected("a timeline action"))
    }

    // ------------------------------------------------------------ runs

    fn run_decl(&mut self, start: Span) -> P<RunDecl> {
        let name = self.name("a run name")?;
        self.expect_word("of")?;
        let model = self.name("a model name")?;
        let presentation = if self.eat_word("with") { Some(self.name("a presentation name")?) } else { None };
        let mut run = RunDecl {
            name,
            model,
            presentation,
            params: vec![],
            inputs: vec![],
            config: vec![],
            until: None,
            learner: vec![],
            expects: vec![],
            span: start,
        };
        enum RunItem {
            Params(Vec<(Name, Expr)>),
            Inputs(Vec<(Name, Expr, Option<Expr>)>),
            Config(Vec<(Name, Expr)>),
            Until(Expr),
            Learner(Vec<LearnerStep>),
            Expect(Vec<Expect>),
        }
        let assign = |p: &mut Self| -> P<Vec<(Name, Expr)>> {
            let n = p.name("a name")?;
            p.expect_punct("=")?;
            Ok(vec![(n, p.expr()?)])
        };
        let items = self.block(|p| {
            if p.eat_word("param") {
                let v = if p.is_punct("{") { p.block(assign)? } else { assign(p)? };
                return Ok(vec![RunItem::Params(v)]);
            }
            if p.eat_word("input") {
                // `x = v` from the start, `x = v at τ` from an instant (D-051).
                let one = |p: &mut Self| -> P<Vec<(Name, Expr, Option<Expr>)>> {
                    let n = p.name("an input name")?;
                    p.expect_punct("=")?;
                    let v = p.expr()?;
                    let at = if p.eat_word("at") { Some(p.expr()?) } else { None };
                    Ok(vec![(n, v, at)])
                };
                let v = if p.is_punct("{") { p.block(one)? } else { one(p)? };
                return Ok(vec![RunItem::Inputs(v)]);
            }
            if p.eat_word("config") {
                let v = if p.is_punct("{") { p.block(assign)? } else { assign(p)? };
                return Ok(vec![RunItem::Config(v)]);
            }
            if p.eat_word("until") {
                return Ok(vec![RunItem::Until(p.expr()?)]);
            }
            if p.eat_word("learner") {
                return Ok(vec![RunItem::Learner(p.block(|p| Self::one(p.learner_step()))?)]);
            }
            if p.eat_word("expect") {
                let v = if p.is_punct("{") { p.block(|p| Self::one(p.expect()))? } else { vec![p.expect()?] };
                return Ok(vec![RunItem::Expect(v)]);
            }
            Err(p.unexpected("`param`, `input`, `config`, `until`, `learner` or `expect`"))
        })?;
        for it in items {
            match it {
                RunItem::Params(v) => run.params.extend(v),
                RunItem::Inputs(v) => run.inputs.extend(v),
                RunItem::Config(v) => run.config.extend(v),
                RunItem::Until(e) => run.until = Some(e),
                RunItem::Learner(v) => run.learner.extend(v),
                RunItem::Expect(v) => run.expects.extend(v),
            }
        }
        run.span = self.since(start);
        Ok(run)
    }

    fn learner_step(&mut self) -> P<LearnerStep> {
        let start = self.span();
        self.expect_word("at")?;
        let at = self.expr()?;
        self.expect_punct(":")?;
        let action = if self.eat_word("continue") {
            LearnerAction::Continue
        } else if self.eat_word("set") {
            let mut control = vec![];
            while !self.is_punct("=") {
                control.push(self.any_name("a control")?);
            }
            self.bump();
            LearnerAction::Set { control, value: self.expr()? }
        } else if self.eat_word("press") {
            LearnerAction::Press(self.name("an event name")?)
        } else {
            return Err(self.unexpected("`set`, `press` or `continue`"));
        };
        Ok(LearnerStep { at, action, span: self.since(start) })
    }

    fn expect(&mut self) -> P<Expect> {
        let start = self.span();
        let word = |t: &Tok| matches!(t, Tok::Ident(w) if !is_reserved(w) && w != "of");
        if word(self.peek()) && word(self.peek_at(1)) {
            let mut words = vec![];
            while matches!(self.peek(), Tok::Ident(_)) {
                words.push(self.any_name("a word")?);
            }
            return Ok(Expect::Outcome { words, span: self.since(start) });
        }
        let e = self.expr_no_in()?;
        if self.eat_word("in") {
            let interval = self.interval()?;
            return Ok(Expect::In { expr: e, interval, span: self.since(start) });
        }
        let (lhs, rhs) = match e.kind {
            ExprKind::Compare(l, mut rest) if rest.len() == 1 && rest[0].0 == CmpOp::Eq => (*l, rest.remove(0).1),
            _ => return Err(Diag::new("SX-E02", "an expectation is `a == b [within tol | exactly]` or `a in I`", e.span)),
        };
        let tol = if self.eat_word("exactly") {
            Tolerance::Exact
        } else if self.eat_word("within") {
            if self.eat_word("rel") {
                Tolerance::Rel(self.expr()?)
            } else {
                Tolerance::Abs(self.expr()?)
            }
        } else {
            Tolerance::None
        };
        Ok(Expect::Compare { lhs, rhs, tol, span: self.since(start) })
    }
}

/// Where a word that starts a declaration, an operation, a control or a representation
/// belongs, for diagnostics on a word found in the wrong block: the question a reader new to
/// the language most often has.
fn belongs(word: &str) -> Option<&'static str> {
    Some(match word {
        "param" | "state" | "derived" | "discrete" | "flow" | "event" | "process" | "constraint" | "collection" | "relation" | "contains" | "object" => {
            "This declaration belongs directly inside a `model Name { ... }`"
        }
        "der" => "`der(x) = ...` gives a rate of change: it belongs in the model's `flow { ... }` block, and `x` is declared in `state { ... }`",
        "set" | "create" | "destroy" | "connect" | "disconnect" => {
            "An operation like this happens at an instant: it belongs in an event's braces, `event name on falling(...) { set x = 1 }`"
        }
        "slider" | "number_input" | "toggle" | "button" | "label" => {
            "A control belongs in a presentation, inside `panel name { ... }` (or inside a view's braces)"
        }
        "marker" | "arrow" | "segment" | "polyline" | "polygon" | "circle" | "ellipse" | "arc" | "trace" | "function_graph" | "series_plot" | "histogram" | "axes" | "grid"
        | "formula" | "equation" | "table" | "group" => "A representation is drawn in a view: it belongs inside `view name: plot(...) { ... }` or `view name: spatial(...) { ... }` of a presentation",
        "view" | "panel" | "observe" | "instrument" | "permit" | "timeline" | "layout" => "This belongs directly inside a `presentation Name for Model { ... }`",
        "scene" | "beat" | "narrate" | "explore" | "hold" => "This belongs in a presentation's `timeline { ... }` (a lesson)",
        "expect" | "until" | "config" => "This belongs in a test case, `run name of Model with Presentation { ... }`",
        "model" | "presentation" | "run" | "space" => "This starts a new top-level item: close the braces of the item before it first",
        _ => return None,
    })
}
