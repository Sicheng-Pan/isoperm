use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bimap::BiMap;
use itertools::{zip, Itertools};

use crate::statement::{group_constraints, Constraint, Variable};

#[derive(Clone, Debug)]
pub(crate) struct StatementEnumerator {
    environment: BiMap<Variable, Variable>,
    local: Vec<(Vec<Variable>, Vec<Variable>)>,
    group: Vec<GroupEnumerator>,
    unconfined: Option<Vec<GroupEnumerator>>,
    stage: Option<usize>,
}

impl StatementEnumerator {
    pub(crate) fn new<T: Eq + Hash>(
        source_constraints: Vec<Constraint>,
        source_variables: &HashMap<Variable, T>,
        target_constraints: Vec<Constraint>,
        target_variables: &HashMap<Variable, T>,
    ) -> Result<Self, String> {
        // Assume that variables with the same name have the same type.
        // Introduce all global variables to the environment.
        let environment = source_variables
            .iter()
            .chain(target_variables.iter())
            .filter_map(|(&v, _)| match v {
                Variable::Global(_) => Some((v, v)),
                _ => None,
            })
            .collect();
        // Collect local variables.
        let local = Variable::group_local_by_type(source_variables)
            .into_iter()
            .chain(Variable::group_local_by_type(target_variables))
            .into_group_map()
            .into_iter()
            .map(|(_, v)| {
                v.into_iter()
                    .collect_tuple()
                    .filter(|(source, target)| source.len() == target.len())
                    .ok_or(String::from("Local variable mismatch."))
            })
            .collect::<Result<_, _>>()?;
        // Transform constraint groups to enumerators.
        let source_groups = group_constraints(source_constraints, source_variables)?;
        let target_groups = group_constraints(target_constraints, target_variables)?;
        let group = source_groups
            .into_iter()
            .chain(target_groups.into_iter())
            .into_group_map()
            .into_iter()
            .map(|(k, v)| {
                v.into_iter()
                    .collect_tuple()
                    .filter(|(source, target)| source.len() == target.len())
                    .ok_or(format!("Constraint {:?} mismatch.", k.0))
                    .map(|(s, t)| GroupEnumerator::new(s, t))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { environment, local, group, unconfined: None, stage: Some(0) })
    }

    // Match constraint groups if needed and check if they are matched.
    fn advance_group(&mut self) -> bool {
        if let Some(mut index) = self.stage {
            while let Some(focus) = self.group.get_mut(index) {
                if focus.advance(&mut self.environment) {
                    index += 1;
                } else if index == 0 {
                    self.stage = None;
                    return false;
                } else {
                    focus.reset(&mut self.environment);
                    index -= 1;
                }
            }
            self.stage = Some(index);
            true
        } else {
            false
        }
    }

    // Generate unconfined groups if needed, and check if there is one.
    fn generate_unconfined(&mut self) -> bool {
        match (self.stage, &self.unconfined) {
            (_, Some(_)) => true,
            (Some(index), None) if index == self.group.len() => {
                self.unconfined = Some(
                    self.local
                        .clone()
                        .into_iter()
                        .filter_map(|(s, t)| {
                            let source_remaining = s
                                .into_iter()
                                .filter(|v| !self.environment.contains_right(v))
                                .map(|v| Constraint::new(0, vec![v]))
                                .collect_vec();
                            let target_remaining = t
                                .into_iter()
                                .filter(|v| !self.environment.contains_left(v))
                                .map(|v| Constraint::new(0, vec![v]))
                                .collect_vec();
                            (!source_remaining.is_empty() && !target_remaining.is_empty())
                                .then(|| GroupEnumerator::new(source_remaining, target_remaining))
                        })
                        .collect(),
                );
                true
            }
            _ => false,
        }
    }

    // Match unconfined variables if needed and check if they are matched.
    fn advance_unconfined(&mut self) -> bool {
        if let (Some(mut index), Some(free)) = (self.stage, &mut self.unconfined) {
            while let Some(focus) = free.get_mut(index - self.group.len()) {
                if focus.advance(&mut self.environment) {
                    index += 1;
                } else if index == self.group.len() {
                    self.stage = (index > 0).then(|| index - 1);
                    self.unconfined = None;
                    return false;
                } else {
                    focus.reset(&mut self.environment);
                    index -= 1;
                }
            }
            if index == 0 {
                self.stage = None;
            } else {
                self.stage = Some(index - 1);
            }
            true
        } else {
            false
        }
    }
}

impl Iterator for StatementEnumerator {
    type Item = BiMap<Variable, Variable>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.stage.is_some() {
            if self.advance_group() && self.generate_unconfined() && self.advance_unconfined() {
                return Some(self.environment.clone());
            }
        }
        None
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
    fn new(source_group: Vec<Constraint>, target_group: Vec<Constraint>) -> Self {
        Self {
            choices: vec![target_group.clone()],
            stage: Vec::new(),
            source: source_group,
            target: target_group.into_iter().collect(),
        }
    }

    // Reset the group enumerator and remove the bindings it created in the
    // environment.
    fn reset(&mut self, environment: &mut BiMap<Variable, Variable>) {
        self.stage.drain(..).for_each(|(focus, commit)| {
            self.target.insert(focus);
            commit.left_values().for_each(|t| {
                environment.remove_by_left(t);
            });
        });
        self.choices.clear();
        self.choices.push(self.target.clone().into_iter().collect());
    }

    // Find the succeeding bindings for the group and commit them to the
    // environment.
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
                        // Some(Some((u, v))) if u and v are not bind to any variable in the
                        // environment Some(None) if u or v are already bind
                        // to other variables in the environment
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
                    commit.left_values().for_each(|t| {
                        environment.remove_by_left(t);
                    });
                }
            }
        }
        false
    }
}
