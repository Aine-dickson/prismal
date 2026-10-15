//! Expressions read back from JSON as the same forms, including the forms of objects and
//! collections (D-055), which share field names with others.

use prismal_ir::{Agg, Builtin, Expr};

#[test]
fn member_and_aggregate_forms_round_trip() {
    let b = |e: Expr| Box::new(e);
    let forms = vec![
        Expr::Var { var: "b".into() },
        Expr::Part { part: "M.part.ball".into() },
        Expr::Item { item: "M.part.row".into(), index: b(Expr::Num { num: 2.0, unit: None }) },
        Expr::Field { field: "M.Ball.pos".into(), of: b(Expr::Var { var: "b".into() }) },
        Expr::Aggregate { aggregate: Agg::Max, var: "b".into(), over: "M.part.row".into(), body: Some(b(Expr::Ref { r#ref: "x".into() })), filter: None, by: None },
        Expr::Aggregate { aggregate: Agg::First, var: "b".into(), over: "M.part.row".into(), body: Some(b(Expr::Ref { r#ref: "x".into() })), filter: None, by: Some(b(Expr::Ref { r#ref: "x".into() })) },
        Expr::Aggregate { aggregate: Agg::Count, var: "_".into(), over: "M.part.row".into(), body: None, filter: None, by: None },
        Expr::Aggregate {
            aggregate: Agg::Sum,
            var: "o".into(),
            over: "M.part.row".into(),
            body: Some(b(Expr::Ref { r#ref: "x".into() })),
            filter: Some(b(Expr::bin(prismal_ir::BinOp::Ne, Expr::Var { var: "o".into() }, Expr::Var { var: "b".into() }))),
            by: None,
        },
        Expr::Builtin { builtin: Builtin::Index },
    ];
    for e in forms {
        let json = serde_json::to_string(&e).unwrap();
        let back: Expr = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e, "{json}");
    }
}
