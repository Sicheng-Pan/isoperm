use crate::wrapper::Isoperm;
use crate::wrapper::Var::*;

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
