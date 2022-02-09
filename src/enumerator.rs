use crate::statement::{Constraint, new_statement, Statement, Variable};
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub(crate) struct Enumerator {
    left: Statement,
    right: Statement,
}

impl Enumerator {
    pub(crate) fn new<T: Eq + Hash>(
        left_constraints: Vec<Constraint>, left_variables: &HashMap<Variable, T>,
        right_constraints: Vec<Constraint>, right_variables: &HashMap<Variable, T>,
    ) -> Result<Self, String> {
        let (left_statement, left_constraint_types) = new_statement(left_constraints, left_variables)?;
        let (right_statement, right_constraint_types) = new_statement(right_constraints, right_variables)?;
        todo!()
    }
}
