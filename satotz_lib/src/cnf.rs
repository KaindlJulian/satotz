use crate::clause::Clause;
use crate::literal::{Literal, Variable};
use crate::parse::parse_dimacs_cnf;
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct CNF {
    clauses: Vec<Clause>,
    variables: Vec<Variable>,
}

impl CNF {
    pub fn empty() -> CNF {
        CNF::default()
    }

    /// Creates a cnf formula from a string in dimacs cnf format
    pub fn from_dimacs(input: &str) -> CNF {
        let clauses = parse_dimacs_cnf(input).expect("parsing error").1;
        CNF::from_clauses(&clauses)
    }

    /// Creates a cnf formula from a file in dimacs cnf format
    pub fn from_file(file: PathBuf) -> CNF {
        CNF::from_dimacs(std::fs::read_to_string(file).expect("fs error").as_str())
    }

    pub fn from_file_str(file: &str) -> CNF {
        CNF::from_file(PathBuf::from(file))
    }

    /// Creates a cnf formula from literals
    pub fn from_clauses(clauses: &[Vec<i32>]) -> CNF {
        let max_var = clauses
            .iter()
            .flat_map(|c| c.iter())
            .map(|l| l.abs())
            .max()
            .unwrap_or(0) as u32;

        CNF {
            clauses: clauses
                .iter()
                .map(|c| {
                    Clause::from_lit_vec(
                        c.iter()
                            .map(|l| Literal::from_dimacs(*l))
                            .collect::<Vec<Literal>>(),
                    )
                })
                .collect(),
            variables: (0..max_var)
                .map(Variable::from_index)
                .collect::<Vec<Variable>>(),
        }
    }

    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    pub fn variables(&self) -> Vec<Variable> {
        self.variables.clone()
    }

    pub fn clauses(&self) -> Vec<Clause> {
        self.clauses.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_clauses() {
        let clauses = vec![vec![1], vec![1, -2, -3], vec![4, 5]];
        let cnf = CNF::from_clauses(&clauses);
        assert_eq!(3, cnf.clauses.len());
        assert_eq!(5, cnf.variables.len());
        assert_eq!(
            cnf.clauses
                .iter()
                .flat_map(|c| c.literals().iter().map(|l| l.as_dimacs_integer()))
                .collect::<Vec<i32>>(),
            clauses
                .iter()
                .flat_map(|c| c.iter().cloned())
                .collect::<Vec<i32>>()
        )
    }

    #[test]
    fn test_variable_count() {
        let cnf = CNF::from_clauses(&[vec![1, 2, 3], vec![10]]);

        // the formula should contain all variables. also variables between 3 and 10
        assert_eq!(10, cnf.variables.len());
        assert_eq!(
            (0..10).collect::<Vec<_>>(),
            cnf.variables.iter().map(|v| v.index()).collect::<Vec<_>>()
        );
    }
}
