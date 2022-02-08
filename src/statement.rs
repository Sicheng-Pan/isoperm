use itertools::Itertools;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Variable {
    Expr(usize),
    Global(usize),
    Local(usize),
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (Variable::Expr(_), Variable::Expr(_)) => true,
            (Variable::Global(one), Variable::Global(other)) => one == other,
            (Variable::Local(one), Variable::Local(other)) => one == other,
            (_, _) => false
        }
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Constraint {
    signature: Variable,
    argument: Vec<Variable>,
}

impl Constraint {
    pub(crate) fn argument(&self) -> &Vec<Variable> {
        &self.argument
    }

    pub(crate) fn argument_types<'a, T: Eq + Hash>(
        &self,
        variable_type: &'a HashMap<Variable, T>,
    ) -> Result<Vec<&'a T>, String> {
        self.argument
            .iter()
            .map(|v| {
                variable_type
                    .get(v)
                    .ok_or(format!("Variable {:?} has undeclared type.", v))
            })
            .collect()
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        &self.signature == &other.signature
    }
}

impl Eq for Constraint {}

impl Hash for Constraint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.signature, state)
    }
}

pub(crate) type Statement = HashMap<Constraint, Vec<Constraint>>;
