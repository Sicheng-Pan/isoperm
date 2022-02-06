use itertools::Itertools;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Variable {
    Any,
    Named(usize),
}

impl<T: Into<Option<usize>>> From<T> for Variable {
    fn from(var: T) -> Self {
        var.into().map_or(Self::Any, Self::Named)
    }
}

#[derive(Clone, Debug)]
struct Constraint<T> {
    signature: (Variable, Vec<T>),
    argument: Vec<Variable>,
}

impl<T> Constraint<T> {
    fn new(signature: (Variable, Vec<T>), argument: Vec<Variable>) -> Self {
        Constraint {
            signature,
            argument,
        }
    }
}

impl<T: PartialEq> PartialEq for Constraint<T> {
    fn eq(&self, other: &Self) -> bool {
        &self.signature == &other.signature
    }
}

impl<T: PartialEq + Eq> Eq for Constraint<T> {}

impl<T: Hash> Hash for Constraint<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.signature, state)
    }
}

pub struct Statement<T>(HashMap<Constraint<T>, Vec<Constraint<T>>>);

impl<U, V, S, T> FromIterator<(U, S)> for Statement<T>
where
    U: Into<Variable>,
    V: Into<Variable>,
    S: IntoIterator<Item = (V, T)>,
    T: Clone + Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = (U, S)>>(iter: I) -> Self {
        Statement(
            iter.into_iter()
                .map(|(u, s)| {
                    let (args, tys) = s.into_iter().map(|(a, t)| (a.into(), t)).unzip();
                    Constraint::new((u.into(), tys), args)
                })
                .into_group_map_by(|c| c.clone()),
        )
    }
}
