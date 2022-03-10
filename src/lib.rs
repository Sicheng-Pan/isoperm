//! A simple implementation for constrained permutations.
//!
//! A constraint here is viewed as a globally identifiable function that takes
//! some arguments with fixed types. For example, `f(int, bool)` can be a
//! constraint that takes in an integer value and a boolean value, with some
//! unknown return value. If we have a bag of constraints where each constraint
//! is applied by a sequence of variables, then we have a bag of unknown return
//! values. Given two bags of constraints and the sets of variables used by
//! each, we would like to find all potential mapping of variables, such that
//! the two bags of constraints can be evaluated to the same bag of results
//! under such mappings.

mod enumerator;
mod statement;
pub mod wrapper;

#[cfg(test)]
mod tests;
