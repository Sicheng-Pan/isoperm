use itertools::Itertools;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Variable {
    Expr(usize),
    Global(usize),
    Local(usize),
}

#[derive(Clone, Debug)]
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

    pub(crate) fn argument_types<'s, T: Eq + Hash>(
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

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        &self.0 == &other.0
    }
}

impl Eq for Constraint {}

impl Hash for Constraint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.0, state)
    }
}

pub(crate) type Statement = HashMap<Constraint, Vec<Constraint>>;

pub(crate) fn new_statement<T: Eq + Hash>(
    constraints: Vec<Constraint>,
    variables: &HashMap<Variable, T>,
) -> Result<(Statement, HashMap<Constraint, Vec<&T>>), String> {
    constraints
        .into_iter()
        .map(|c| c.argument_types(variables).map(|tys| (c, tys)))
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .into_group_map_by(|(c, _)| c.clone())
        .into_iter()
        .map(|(s, cts)| {
            let (cs, mut ts): (Vec<_>, Vec<_>) = cts.into_iter().unzip();
            ts.iter()
                .all_equal()
                .then(|| ((s.clone(), cs), (s.clone(), ts.pop().unwrap())))
                .ok_or(format!("Constraint {} has conflicting types.", s.signature()))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|ct| ct.into_iter().unzip())
}
