use crate::literal::{Literal, Variable};
use std::ops::Not;

/// Possible assignment values for a variable
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssignedValue {
    True,
    False,
    Unknown,
}

impl Not for AssignedValue {
    type Output = AssignedValue;

    fn not(self) -> Self::Output {
        use AssignedValue::*;
        match self {
            True => False,
            False => True,
            Unknown => Unknown,
        }
    }
}

/// Holds assignments to variables
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct VariableAssignment {
    /// maps variables to assignments, indexed by the variables index
    partial: Vec<AssignedValue>,
}

impl VariableAssignment {
    pub fn resize(&mut self, var_count: usize) {
        self.partial.resize(var_count, AssignedValue::Unknown);
    }

    pub fn assign_true(&mut self, lit: Literal) {
        self.partial[lit.variable().index() as usize] = if lit.is_positive() {
            AssignedValue::True
        } else {
            AssignedValue::False
        }
    }

    pub fn assign_unknown(&mut self, var: Variable) {
        self.partial[var.index() as usize] = AssignedValue::Unknown;
    }

    pub fn value(&self, var: Variable) -> AssignedValue {
        self.partial[var.index() as usize]
    }

    pub fn literal_value(&self, lit: Literal) -> AssignedValue {
        let variable_value = self.value(lit.variable());
        if lit.is_positive() {
            variable_value
        } else {
            !variable_value
        }
    }

    /// Returns the literals that are assigned
    pub fn partial(&self) -> Vec<Literal> {
        let mut partial = self
            .partial
            .iter()
            .enumerate()
            .filter(|(_, v)| **v != AssignedValue::Unknown)
            .map(|(k, v)| (k, *v == AssignedValue::True))
            .map(|(k, v)| Literal::from_index(k as u32, v))
            .collect::<Vec<_>>();
        partial.sort();
        partial
    }

    pub fn unassigned(&self) -> Vec<Variable> {
        self.partial
            .iter()
            .enumerate()
            .filter(|(_, v)| **v == AssignedValue::Unknown)
            .map(|(k, _)| Variable::from_index(k as u32))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value_for_variable() {
        let mut assignments = VariableAssignment::default();
        assignments.resize(1);

        let lit = Literal::from_dimacs(1);
        let var = lit.variable();

        assert_eq!(AssignedValue::Unknown, assignments.value(var));
        assignments.assign_true(lit);
        assert_eq!(AssignedValue::True, assignments.value(var));
        assignments.assign_true(!lit);
        assert_eq!(AssignedValue::False, assignments.value(var));
        assignments.assign_unknown(var);
        assert_eq!(AssignedValue::Unknown, assignments.value(var));
    }

    #[test]
    fn test_get_value_for_literal() {
        let mut assignments = VariableAssignment::default();
        assignments.resize(1);

        let a = Literal::from_dimacs(1);

        // a = ?
        assert_eq!(AssignedValue::Unknown, assignments.literal_value(a));
        assert_eq!(AssignedValue::Unknown, assignments.literal_value(!a));
        // a = true
        assignments.assign_true(a);
        assert_eq!(AssignedValue::True, assignments.literal_value(a));
        assert_eq!(AssignedValue::False, assignments.literal_value(!a));
        // !a = true
        assignments.assign_true(!a);
        assert_eq!(AssignedValue::False, assignments.literal_value(a));
        assert_eq!(AssignedValue::True, assignments.literal_value(!a));
    }

    #[test]
    fn test_partial() {
        let mut assignments = VariableAssignment::default();
        assignments.resize(5);

        assignments.assign_true(Literal::from_dimacs(1));
        assignments.assign_true(Literal::from_dimacs(-2));
        assignments.assign_true(Literal::from_dimacs(-3));
        assignments.assign_true(Literal::from_dimacs(4));
        assignments.assign_unknown(Literal::from_dimacs(5).variable());

        assert_eq!(
            assignments.partial(),
            vec![
                Literal::from_dimacs(1),
                Literal::from_dimacs(-2),
                Literal::from_dimacs(-3),
                Literal::from_dimacs(4)
            ]
        )
    }
}