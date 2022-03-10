use crate::enumerator::StatementEnumerator;
use crate::statement::{Constraint, Variable};
use bimap::BiMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

/// # The wrapper variable enum.
/// There are three types of variables:
/// - Expression: An expression is considered to have unknown value. It could be
///   matched to anything. It is not considered as a concrete variable, as a
///   result of which it will not bind to anything.
/// - Global: An global variable is considered to have known value. It could
///   only match to itself and binds to itself.
/// - Local: An local variable is considered to have unknown value. It could
///   match and bind to another local variable of the same type.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Var<U, V = U, W = U>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    // An expression variable could potentially be matched to anything.
    Expr(W),
    // A global variable can only be matched to itself.
    Global(V),
    // A local variable can only be matched to other local variables.
    Local(U),
}

impl<U, V, W> Var<U, V, W>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    fn transform(&self, signature: usize) -> Variable {
        match &self {
            Var::Expr(_) => Variable::Expr(signature),
            Var::Global(_) => Variable::Global(signature),
            Var::Local(_) => Variable::Local(signature),
        }
    }
}

/// # The wrapper permutation struct.
/// In order to construct an iterator of all potential permutations, first
/// construct an instance of `Isoperm` class by provide the two
/// bags of constraints and the variables used by each of them. Then call
/// `result()` to get the actual iterator.
pub struct Isoperm<U, V = U, W = U>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    source_translation: BiMap<Variable, Var<U, V, W>>,
    target_translation: BiMap<Variable, Var<U, V, W>>,
    permutation: StatementEnumerator,
}

impl<U, V, W> Isoperm<U, V, W>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    pub fn new<R, S, T>(
        source_constraints: S,
        source_variables: HashMap<Var<U, V, W>, T>,
        target_constraints: S,
        target_variables: HashMap<Var<U, V, W>, T>,
    ) -> Result<Self, String>
    where
        R: Eq + Hash,
        S: IntoIterator<Item = (R, Vec<Var<U, V, W>>)>,
        T: Eq + Hash,
    {
        let source_native_variables =
            Isoperm::transform_variables(source_variables, &BiMap::new())?;
        let target_native_variables =
            Isoperm::transform_variables(target_variables, &source_native_variables)?;
        let (source_types, source_translation) = Isoperm::split_mapping(source_native_variables);
        let (target_types, target_translation) = Isoperm::split_mapping(target_native_variables);
        let mut constraint_record = HashMap::new();
        let source_native_constraints = Isoperm::transform_constraints(
            source_constraints,
            &source_translation,
            &mut constraint_record,
        )?;
        let target_native_constraints = Isoperm::transform_constraints(
            target_constraints,
            &target_translation,
            &mut constraint_record,
        )?;
        let permutation = StatementEnumerator::new(
            source_native_constraints,
            &source_types,
            target_native_constraints,
            &target_types,
        )?;
        Ok(Self { source_translation, target_translation, permutation })
    }

    fn transform_variables<T>(
        variables: HashMap<Var<U, V, W>, T>,
        reference: &BiMap<(Variable, T), Var<U, V, W>>,
    ) -> Result<BiMap<(Variable, T), Var<U, V, W>>, String>
    where
        T: Eq + Hash,
    {
        variables
            .into_iter()
            .enumerate()
            .map(|(signature, (v, t))| match reference.get_by_right(&v) {
                Some((vr, tr)) if matches!(&v, Var::Global(_)) => (&t == tr)
                    .then(|| ((vr.clone(), t), v))
                    .ok_or(String::from("Global variable type mismatch.")),
                _ => Ok(((v.transform(signature), t), v)),
            })
            .collect()
    }

    fn transform_constraints<R, S>(
        constraints: S,
        variables: &BiMap<Variable, Var<U, V, W>>,
        record: &mut HashMap<R, usize>,
    ) -> Result<Vec<Constraint>, String>
    where
        R: Eq + Hash,
        S: IntoIterator<Item = (R, Vec<Var<U, V, W>>)>,
    {
        constraints.into_iter().try_fold(Vec::new(), |mut transformed, (signature, arguments)| {
            arguments
                .into_iter()
                .map(|v| variables.get_by_right(&v).map(|vl| vl.clone()))
                .collect::<Option<_>>()
                .map(|vs| {
                    let frame = record.len();
                    transformed
                        .push(Constraint::new(*record.entry(signature).or_insert(frame), vs));
                    transformed
                })
                .ok_or(String::from("Undeclared variable in constraint."))
        })
    }

    fn split_mapping<T>(
        translation: BiMap<(Variable, T), Var<U, V, W>>,
    ) -> (HashMap<Variable, T>, BiMap<Variable, Var<U, V, W>>)
    where
        T: Eq + Hash,
    {
        translation.into_iter().map(|((vl, t), vr)| ((vl.clone(), t), (vl, vr))).unzip()
    }

    /// Returns the iterator of all possible permutations.
    pub fn result(&mut self) -> Isopermutation<U, V, W> {
        Isopermutation {
            source: &self.source_translation,
            target: &self.target_translation,
            perm: &mut self.permutation,
        }
    }
}

/// The wrapper permutation iterator struct.
pub struct Isopermutation<'t, U, V = U, W = U>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    source: &'t BiMap<Variable, Var<U, V, W>>,
    target: &'t BiMap<Variable, Var<U, V, W>>,
    perm: &'t mut StatementEnumerator,
}

impl<'t, U, V, W> Iterator for Isopermutation<'t, U, V, W>
where
    U: Eq + Hash + PartialEq,
    V: Eq + Hash + PartialEq,
    W: Eq + Hash + PartialEq,
{
    type Item = BiMap<&'t Var<U, V, W>, &'t Var<U, V, W>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.perm.next().map(|binding| {
            binding
                .into_iter()
                .map(|(t, s)| {
                    (self.target.get_by_left(&t).unwrap(), self.source.get_by_left(&s).unwrap())
                })
                .collect()
        })
    }
}
