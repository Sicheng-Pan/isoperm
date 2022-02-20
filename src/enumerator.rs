use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bimap::BiMap;
use itertools::{Itertools, zip};

use crate::statement::{Constraint, group_constraints, Variable};

#[derive(Clone, Debug)]
pub(crate) struct StatementEnumerator<I: Iterator<Item=Vec<Variable>>> {
    environment: BiMap<Variable, Variable>,
    focus: HashSet<Variable>,
    group: Vec<GroupEnumerator>,
    unconfined: Option<I>,
}

impl<I: Iterator<Item=Vec<Variable>>> StatementEnumerator<I> {
    pub(crate) fn new<T: Eq + Hash>(
        left_constraints: Vec<Constraint>,
        left_variables: &HashMap<Variable, T>,
        right_constraints: Vec<Constraint>,
        right_variables: &HashMap<Variable, T>,
    ) -> Result<Self, String> {
        // TODO: CHECK VARIABLES
        let left_groups = group_constraints(left_constraints, left_variables)?;
        let right_groups = group_constraints(right_constraints, right_variables)?;
        let _ = left_groups
            .into_iter()
            .chain(right_groups.into_iter())
            .into_group_map()
            .into_iter()
            .map(|(k, v)| {
                v.into_iter()
                    .collect_tuple()
                    .ok_or(format!("Constraint {:?} has no matching group.", k.0))
                    .map(|(l, r)| GroupEnumerator::new(l, r))
            })
            .collect::<Result<Vec<_>, String>>()?;
        todo!()
        // Introduce all global variables to the environment.
    }
}

#[derive(Clone, Debug)]
struct GroupEnumerator {
    choices: Vec<Vec<Constraint>>,
    stage: Vec<(Constraint, BiMap<Variable, Variable>)>,
    target: HashSet<Constraint>,
    source: Vec<Constraint>,
}

impl GroupEnumerator {
    fn new(source_group: Vec<Constraint>, target_group: Vec<Constraint>) -> Result<Self, String> {
        (source_group.len() == target_group.len())
            .then(|| Self {
                choices: vec![target_group.clone()],
                stage: Vec::new(),
                source: source_group,
                target: target_group.into_iter().collect(),
            })
            .ok_or(String::from("Constraint groups have mismatched length."))
    }

    // Initialize the group enumerator and remove the bindings it created in the environment.
    fn initialize(&mut self, environment: &mut BiMap<Variable, Variable>) {
        self.stage.drain(..).for_each(|(focus, commit)| {
            self.target.insert(focus);
            commit.left_values().for_each(|t| { environment.remove_by_left(t); });
        });
        self.choices.clear();
        self.choices.push(self.target.clone().into_iter().collect());
    }

    // Find the succeeding bindings for the group and commit them to the environment.
    fn advance(&mut self, environment: &mut BiMap<Variable, Variable>) -> bool {
        use Variable::*;
        while let Some(candidates) = self.choices.last_mut() {
            if let Some(focus) = candidates.pop() {
                let correspondence = self.source.get(self.choices.len() - 1).unwrap().clone();
                if let Some(binding) = zip(focus.argument(), correspondence.argument())
                    // Ignore bindings with expression variables.
                    .filter(|&bind| !matches!(bind, (&Expr(_), _) | (_, &Expr(_))))
                    .filter_map(|(u, v)| {
                        // Assume that global variables are self-bind in the environment
                        // Some(Some((u, v))) if u and v are not bind to any variable in the environment
                        // Some(None) if u or v are already bind to other variables in the environment
                        // None if u and v are already bin to each other in the environment
                        match (environment.get_by_left(u), environment.get_by_right(v)) {
                            (None, None) => Some(Some((*u, *v))),
                            (q, p) => (u != p.unwrap_or(u) || v != q.unwrap_or(v)).then(|| None),
                        }
                    })
                    .try_fold(BiMap::new(), |mut introduced, bind| {
                        // Bind u with v, and abort if there is conflict.
                        bind.filter(|&(u, v)| introduced.insert_no_overwrite(u, v).is_ok())
                            .map(|_| introduced)
                    })
                {
                    // Commit bindings to the environment and advance in stage.
                    self.target.remove(&focus);
                    self.choices.push(self.target.clone().into_iter().collect());
                    self.stage.push((focus, binding.clone()));
                    environment.extend(binding);
                    if self.target.is_empty() {
                        return true;
                    }
                }
            } else {
                // Undo the last stage.
                self.choices.pop();
                if let Some((focus, commit)) = self.stage.pop() {
                    self.target.insert(focus);
                    commit.left_values().for_each(|t| { environment.remove_by_left(t); });
                }
            }
        }
        false
    }
}
