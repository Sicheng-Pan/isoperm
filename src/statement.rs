use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Variable {
    Expr(usize),
    Global(usize),
    Local(usize),
}

impl Variable {
    pub(crate) fn group_local_by_type<T: Eq + Hash>(
        variables: &HashMap<Variable, T>,
    ) -> HashMap<&T, Vec<Variable>> {
        variables
            .iter()
            .filter_map(|(&v, t)| match v {
                Variable::Local(_) => Some((t, v)),
                _ => None,
            })
            .into_group_map()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Constraint(usize, Vec<Variable>);

impl Constraint {
    pub(crate) fn new(signature: usize, argument: Vec<Variable>) -> Self {
        Self(signature, argument)
    }

    pub(crate) fn signature(&self) -> usize {
        self.0
    }

    pub(crate) fn argument(&self) -> &Vec<Variable> {
        &self.1
    }

    // Lookup argument types given the types of variables.
    fn argument_types<'s, T: Eq + Hash>(
        &self,
        variable_type: &'s HashMap<Variable, T>,
    ) -> Result<Vec<&'s T>, String> {
        self.1
            .iter()
            .map(|v| {
                variable_type
                    .get(v)
                    .ok_or(format!("Variable {:?} has undeclared type.", v))
            })
            .collect()
    }
}

// Group constraints by their signatures and argument types.
pub(crate) fn group_constraints<T: Eq + Hash>(
    constraints: Vec<Constraint>,
    variables: &HashMap<Variable, T>,
) -> Result<HashMap<(usize, Vec<&T>), Vec<Constraint>>, String> {
    constraints
        .into_iter()
        .map(|c| {
            c.argument_types(variables)
                .map(|tys| ((c.signature(), tys), c))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|group| group.into_iter().into_group_map())
}
