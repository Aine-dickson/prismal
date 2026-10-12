//! Lexical analysis (working syntax section 1.5, D-035).

use crate::{Diag, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Num(f64),
    Str(String),
    /// Operators and punctuation, longest match first.
    Punct(&'static str),
    /// A statement end: a newline that does not continue the statement.
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// Whitespace or a comment directly before the token. Units and the product `2π` are
    /// recognized only where this is false.
    pub space_before: bool,
}

/// A comment. Comments on a line of their own may become author notes (D-036).
#[derive(Clone, Debug)]
pub struct Comment {
    pub line: u32,
    pub text: String,
    pub doc: bool,
    pub own_line: bool,
}

const PUNCT: [&str; 28] = [
    "==", "=>", "!=", "<=", ">=", "+=", "->", "..", "+", "-", "*", "/", "^", "=", "<", ">", "|", ".", ",", ":", ";", "(",
    ")", "[", "]", "{", "}", "!",
];

/// Words after which a line continues (working syntax 1.5).
const CONTINUING_WORDS: [&str; 7] = ["and", "or", "not", "if", "then", "else", "otherwise"];

fn continues(t: &Tok) -> bool {
    match t {
        Tok::Punct(p) => !matches!(*p, ")" | "]" | "}" | "|" | ";"),
        Tok::Ident(w) => CONTINUING_WORDS.contains(&w.as_str()),
        _ => false,
    }
}

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

pub fn lex(src: &str) -> (Lexed, Vec<Diag>) {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let mut tokens: Vec<Token> = vec![];
    let mut comments = vec![];
    let mut diags = vec![];
    let (mut i, mut line, mut col) = (0usize, 1u32, 1u32);
    let mut depth = 0i32; // parentheses and brackets: newlines inside them do not end statements
    let mut space = true;
    let mut line_has_token = false;
    let at = |i: usize| chars.get(i).map(|c| c.1);
    let offset = |i: usize| chars.get(i).map(|c| c.0).unwrap_or(src.len());

    while i < chars.len() {
        let c = chars[i].1;
        let start = i;
        let span_at = |i0: usize, i1: usize, line: u32, col: u32| Span { line, col, start: offset(i0), end: offset(i1) };
        if c == '\n' {
            let last = tokens.last().map(|t: &Token| &t.tok);
            let suppressed = depth > 0 || matches!(last, None | Some(Tok::Newline)) || last.map(continues).unwrap_or(false);
            if !suppressed {
                tokens.push(Token { tok: Tok::Newline, span: span_at(i, i + 1, line, col), space_before: space });
            }
            i += 1;
            line += 1;
            col = 1;
            space = true;
            line_has_token = false;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            col += 1;
            space = true;
            continue;
        }
        if c == '/' && at(i + 1) == Some('/') {
            let doc = at(i + 2) == Some('/') && at(i + 3) != Some('/');
            let skip = if doc { 3 } else { 2 };
            let mut j = i + skip;
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            let text: String = chars[i + skip..j].iter().map(|c| c.1).collect();
            let text = text.strip_prefix(' ').unwrap_or(&text).trim_end().to_string();
            comments.push(Comment { line, text, doc, own_line: !line_has_token });
            col += (j - i) as u32;
            i = j;
            space = true;
            continue;
        }
        let tok_line = line;
        let tok_col = col;
        let tok = if c.is_ascii_digit() {
            let mut j = i;
            while at(j).is_some_and(|c| c.is_ascii_digit()) {
                j += 1;
            }
            if at(j) == Some('.') && at(j + 1).is_some_and(|c| c.is_ascii_digit()) {
                j += 1;
                while at(j).is_some_and(|c| c.is_ascii_digit()) {
                    j += 1;
                }
            }
            if matches!(at(j), Some('e' | 'E')) {
                let k = if matches!(at(j + 1), Some('+' | '-')) { j + 2 } else { j + 1 };
                if at(k).is_some_and(|c| c.is_ascii_digit()) {
                    j = k;
                    while at(j).is_some_and(|c| c.is_ascii_digit()) {
                        j += 1;
                    }
                }
            }
            let text = &src[offset(i)..offset(j)];
            i = j;
            Tok::Num(text.parse().expect("digits parse as a number"))
        } else if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while at(j).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                j += 1;
            }
            let text = src[offset(i)..offset(j)].to_string();
            i = j;
            Tok::Ident(text)
        } else if c == '%' {
            // The percent unit (D-077): a unit symbol, never a name or an operator.
            i += 1;
            Tok::Ident("%".into())
        } else if c == '"' {
            let mut j = i + 1;
            let mut s = String::new();
            let mut closed = false;
            while let Some(d) = at(j) {
                if d == '\n' {
                    break;
                }
                j += 1;
                if d == '"' {
                    closed = true;
                    break;
                }
                if d == '\\' {
                    match at(j) {
                        Some('n') => s.push('\n'),
                        Some(e) => s.push(e),
                        None => {}
                    }
                    j += 1;
                    continue;
                }
                s.push(d);
            }
            if !closed {
                diags.push(Diag::new("SX-E01", "unterminated string", span_at(i, j, line, col)));
            }
            i = j;
            Tok::Str(s)
        } else if let Some(p) = PUNCT.iter().find(|p| src[offset(i)..].starts_with(**p)) {
            i += p.chars().count();
            match *p {
                "(" | "[" => depth += 1,
                ")" | "]" => depth = (depth - 1).max(0),
                _ => {}
            }
            Tok::Punct(p)
        } else {
            diags.push(Diag::new("SX-E01", format!("unexpected character `{c}`"), span_at(i, i + 1, line, col)));
            i += 1;
            col += 1;
            space = true;
            continue;
        };
        col += (i - start) as u32;
        tokens.push(Token { tok, span: span_at(start, i, tok_line, tok_col), space_before: space });
        space = false;
        line_has_token = true;
    }
    let end = Span { line, col, start: src.len(), end: src.len() };
    if !matches!(tokens.last().map(|t| &t.tok), None | Some(Tok::Newline)) {
        tokens.push(Token { tok: Tok::Newline, span: end, space_before: true });
    }
    tokens.push(Token { tok: Tok::Eof, span: end, space_before: true });
    (Lexed { tokens, comments }, diags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<Tok> {
        lex(s).0.tokens.into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn numbers_units_and_continuation() {
        use Tok::*;
        assert_eq!(
            toks("g = 9.81 m/s^2\nx = 1e-9 +\n  2"),
            vec![
                Ident("g".into()),
                Punct("="),
                Num(9.81),
                Ident("m".into()),
                Punct("/"),
                Ident("s".into()),
                Punct("^"),
                Num(2.0),
                Newline,
                Ident("x".into()),
                Punct("="),
                Num(1e-9),
                Punct("+"),
                Num(2.0),
                Newline,
                Eof
            ]
        );
    }

    #[test]
    fn newlines_inside_brackets_and_comments() {
        let (l, d) = lex("f(a,\n b) // note\n/// doc\nθ0");
        assert!(d.is_empty());
        let t: Vec<Tok> = l.tokens.into_iter().map(|t| t.tok).collect();
        assert_eq!(t.iter().filter(|t| **t == Tok::Newline).count(), 2);
        assert_eq!(l.comments.len(), 2);
        assert!(!l.comments[0].own_line && !l.comments[0].doc);
        assert!(l.comments[1].own_line && l.comments[1].doc && l.comments[1].text == "doc");
        assert!(t.contains(&Tok::Ident("θ0".into())));
    }
}
