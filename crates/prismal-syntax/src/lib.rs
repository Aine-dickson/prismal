//! The Prismal working syntax (D-028, D-035; `docs/syntax-study/working-syntax.md`).
//!
//! `parse` turns source text into a syntax tree (`ast`); `lower` turns its spaces and models
//! into the semantic IR (`docs/spec/04-ir.md`), with a source map from IR identities back to
//! the text; `check` also runs the model kernel's static checks and reports their
//! diagnostics at the declarations they concern. `format` prints an IR document back as
//! canonical text, and `identity` keeps identities across edits (D-036).

pub mod ast;
pub mod format;
pub mod identity;
pub mod lexer;
pub mod lower;
mod lower_present;
pub mod modules;
pub mod parser;

pub use format::format;
pub use identity::{diff, reconcile, rename, IdentityDiff};
pub use lower::{lower_expr, SourceMap};
pub use modules::{DirModules, FirstParty, MapModules, Modules, NoModules};

use prismal_ir::{Document, Model};
use prismal_kernel::{check_model, CModel};

/// A region of source text: 1-based line and column of its start, byte offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub start: usize,
    pub end: usize,
}

/// A diagnostic from the lexer, parser, lowering or the model kernel.
///
/// Codes: `SX-E01` lexical error, `SX-E02` syntax error, `SX-E03` unknown name, `SX-E04`
/// unknown type or dimension, `SX-E05` unknown unit, `SX-E06` not in the v0 IR,
/// `SX-E07` reserved word used as a name, `SX-E08` form not allowed here, `SX-E09`
/// duplicate declaration; `MK-E..` are the kernel's static diagnostics (MK section 18).
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    /// The IR element a kernel diagnostic is about.
    pub element: Option<String>,
}

impl Diag {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Diag {
        Diag { code, message: message.into(), span, element: None }
    }

    /// `file:line:col: code: message`, then the source line with a marker under the span.
    pub fn render(&self, src: &str, file: &str) -> String {
        let line_text = src.lines().nth(self.span.line.saturating_sub(1) as usize).unwrap_or("");
        let width = if self.span.end > self.span.start {
            src.get(self.span.start..self.span.end).map(|s| s.lines().next().unwrap_or("").chars().count()).unwrap_or(1).max(1)
        } else {
            1
        };
        let pad: String = line_text.chars().take(self.span.col.saturating_sub(1) as usize).map(|c| if c == '\t' { '\t' } else { ' ' }).collect();
        format!(
            "{file}:{}:{}: {}: {}\n    {line_text}\n    {pad}{}",
            self.span.line,
            self.span.col,
            self.code,
            self.message,
            "^".repeat(width)
        )
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}: {}", self.span.line, self.span.col, self.code, self.message)
    }
}

/// Parses source text into a syntax tree. The tree is returned even when there are errors.
pub fn parse(src: &str) -> (ast::File, Vec<Diag>) {
    let (lexed, mut diags) = lexer::lex(src);
    let mut p = parser::Parser::new(&lexed.tokens, &lexed.comments);
    let file = p.file();
    diags.extend(p.diags);
    (file, diags)
}

/// Parses one expression, as written inside `{...}` in a `text` representation (D-076).
pub fn parse_expr(src: &str) -> Result<ast::Expr, String> {
    let (lexed, diags) = lexer::lex(src);
    if let Some(d) = diags.first() {
        return Err(d.message.clone());
    }
    let mut p = parser::Parser::new(&lexed.tokens, &lexed.comments);
    p.whole_expr().map_err(|d| d.message)
}

/// A parsed and lowered source text.
#[derive(Clone, Debug)]
pub struct Compiled {
    pub file: ast::File,
    pub doc: Document,
    pub source_map: SourceMap,
}

impl Compiled {
    pub fn model(&self, name: &str) -> Option<&Model> {
        self.doc.models.iter().find(|m| m.name == name)
    }
}

/// Parses and lowers source text to the IR. Fails with every syntax and lowering diagnostic.
/// `use` finds the first-party modules (D-099).
pub fn compile(src: &str) -> Result<Compiled, Vec<Diag>> {
    compile_with(src, &FirstParty)
}

/// Parses and lowers source text whose `use` lines find their modules in `modules` (D-099).
/// The document holds the shared definitions of every module it uses, so it needs no modules
/// afterwards.
pub fn compile_with(src: &str, modules: &dyn Modules) -> Result<Compiled, Vec<Diag>> {
    let (file, mut diags) = parse(src);
    if !diags.is_empty() {
        return Err(diags);
    }
    let (doc, source_map, d) = lower::lower_with(&file, modules);
    diags.extend(d);
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(Compiled { file, doc, source_map })
}

/// Compiles source text and checks every model with the kernel. Kernel diagnostics are
/// located at the declaration of the element they name.
pub fn check(src: &str) -> Result<(Compiled, Vec<CModel>), Vec<Diag>> {
    let c = compile(src)?;
    let mut diags = vec![];
    let mut out = vec![];
    let linked = prismal_ir::link::link(c.doc.clone());
    for m in &linked.models {
        match check_model(&c.doc.spaces, m) {
            Ok(cm) => out.push(cm),
            Err(ds) => {
                for d in ds {
                    let span = c.source_map.get(&d.element).or_else(|| c.source_map.get(&m.id)).copied().unwrap_or_default();
                    diags.push(Diag { code: d.code, message: d.message, span, element: Some(d.element) });
                }
            }
        }
    }
    if diags.is_empty() {
        Ok((c, out))
    } else {
        Err(diags)
    }
}

/// The program in a Markdown document: the contents of its ```` ```text ```` and
/// ```` ```cases ```` blocks, with every other line blanked so that line numbers match the
/// document.
pub fn markdown_source(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut inside = false;
    for line in md.lines() {
        let fence = line.trim_start().starts_with("```");
        if !inside && fence {
            let lang = line.trim_start().trim_start_matches('`').trim();
            // Blocks in other languages stay blank; their closing fence opens nothing.
            inside = lang == "text" || lang == "cases";
            out.push('\n');
            continue;
        }
        if inside && fence {
            inside = false;
            out.push('\n');
            continue;
        }
        if inside {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// The ```` ```text ```` and ```` ```cases ```` blocks of a Markdown document, each with
/// the line number (1-based) of its first line.
pub fn markdown_blocks(md: &str) -> Vec<(u32, String)> {
    let mut out = vec![];
    let mut cur: Option<(u32, String)> = None;
    let mut other = false;
    for (i, line) in md.lines().enumerate() {
        let fence = line.trim_start().starts_with("```");
        match (&mut cur, fence) {
            (Some(_), true) => out.push(cur.take().unwrap()),
            (Some((_, text)), false) => {
                text.push_str(line);
                text.push('\n');
            }
            (None, true) if other => other = false,
            (None, true) => {
                let lang = line.trim_start().trim_start_matches('`').trim();
                if lang == "text" || lang == "cases" {
                    cur = Some((i as u32 + 2, String::new()));
                } else {
                    other = true;
                }
            }
            (None, false) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_blocks_keep_line_numbers() {
        let md = "# T\n\n```text\nspace Plane = euclidean(2)\n```\n\n```json\n{}\n```\n";
        let s = markdown_source(md);
        assert_eq!(s.lines().nth(3), Some("space Plane = euclidean(2)"));
        assert_eq!(s.lines().filter(|l| !l.is_empty()).count(), 1);
    }

    #[test]
    fn render_marks_the_span() {
        let src = "model M {\n  param g: Acceleration = 9.81 m/s^2 +\n}\n";
        let err = compile(src).unwrap_err();
        let r = err[0].render(src, "m.prismal");
        assert!(r.starts_with("m.prismal:"), "{r}");
        assert!(r.contains('^'), "{r}");
    }
}
