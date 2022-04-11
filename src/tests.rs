use crate::wrapper::Var::*;
use crate::wrapper::{Isoperm, Var};
use bimap::BiMap;
use std::iter::once;

#[test]
fn basic_test() {
    let source_variables = vec![
        (Global(0), false),
        (Local(0), false),
        (Local(1), true),
        (Local(2), true),
        (Expr(0), true),
    ]
    .into_iter()
    .collect();
    let target_variables = vec![
        (Global(0), false),
        (Local(0), true),
        (Local(1), true),
        (Local(2), false),
        (Expr(0), true),
    ]
    .into_iter()
    .collect();
    let source_constraints = vec![("R", vec![Global(0), Local(0)])];
    let target_constraints = vec![("R", vec![Global(0), Local(2)])];
    let mut isoperm =
        Isoperm::new(source_constraints, source_variables, target_constraints, target_variables)
            .unwrap();
    isoperm.result().take(5).for_each(|bind| println!("{:?}", bind));
}

#[test]
fn zero_local_test() {
    let source_variables = once((Global(0), false)).collect();
    let target_variables = once((Global(0), false)).collect();
    let source_constraints: Vec<(String, _)> = vec![];
    let target_constraints = vec![];
    let mut isoperm =
        Isoperm::new(source_constraints, source_variables, target_constraints, target_variables)
            .unwrap();
    isoperm.result().take(5).for_each(|bind: BiMap<&Var<i32>, &Var<i32>>| println!("{:?}", bind));
}
