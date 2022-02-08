use crate::statement::{Constraint, Statement, Variable};
use std::collections::HashMap;
use std::hash::Hash;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub(crate) struct Enumerator {
    left: Statement,
    right: Statement,
}

impl Enumerator {
    pub(crate) fn new<T: Eq + Hash>(
        left: (Vec<Constraint>, &HashMap<Variable, T>),
        right: (Vec<Constraint>, &HashMap<Variable, T>),
    ) -> Result<Enumerator, String> {
        let (left_constraints, left_variables) = left;
        let (right_constraints, right_variables) = right;
        let left_statement = left_constraints
            .into_iter()
            .map(|c| (c.argument_types(left_variables), c))
            .into_group_map();
        todo!()
    }
}
