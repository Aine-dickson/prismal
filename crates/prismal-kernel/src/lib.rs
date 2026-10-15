//! Prismal model kernel (docs/spec/01-model-kernel.md): static checking, dependency
//! analysis, compilation and evaluation of expressions.

pub mod check;
pub mod eval;
pub mod model;
pub mod random;

pub use check::{check_intervention, check_model, compile_expr, show, Diagnostic};
pub use eval::{distance, frame_axes, Arr, CDraw, CExpr, Ctx, Draws, Status, StatusKind, Value};
pub use model::*;
