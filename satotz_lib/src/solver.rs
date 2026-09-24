use crate::assignment::AssignedValue;
use crate::bcp::trail::Trail;
use crate::cnf::CNF;
use crate::literal::Literal;
use crate::resize::Resize;
use crate::search::{search, SearchContext};

#[derive(Default, Debug)]
pub struct Solver {
    search: SearchContext,
    variable_count: usize,
}

impl Solver {
    pub fn from_cnf(cnf: CNF) -> Solver {
        let mut solver = Solver {
            search: Default::default(),
            variable_count: cnf.variable_count(),
        };

        solver.search.use_dlis = true;
        solver.search.resize(solver.variable_count);

        for c in cnf.clauses().iter() {
            solver.add_clause(c.literals());
        }

        solver
    }

    pub fn from_clauses(clauses: Vec<Vec<i32>>) -> Solver {
        Self::from_cnf(CNF::from_clauses(&clauses))
    }

    pub fn without_dlis(mut self) -> Self {
        self.search.use_dlis = false;
        self
    }

    /// Adds a clause to the formula, can break invariants if introducing new variables
    pub fn add_clause(&mut self, clause: &[Literal]) {
        self.search.bcp.add_clause(clause);
    }

    /// Check satisfiability of the formula
    pub fn solve(&mut self) -> bool {
        loop {
            if let Some(result) = search(&mut self.search) {
                return result;
            }
        }
    }

    pub fn solve_with_history(&mut self) -> (Vec<Trail>, bool) {
        let mut history: Vec<Trail> = vec![];
        history.push(self.search.bcp.trail.clone());
        loop {
            let r = search(&mut self.search);
            history.push(self.search.bcp.trail.clone());
            if let Some(result) = r {
                return (history, result);
            }
        }
    }

    pub fn step(&mut self) -> (&mut Self, Option<bool>) {
        let step_result = search(&mut self.search);
        (self, step_result)
    }

    /// Returns the current assignment, literals with unknown value are falsified
    pub fn assignment(&self) -> Vec<Literal> {
        self.search.bcp.assignment.assignment()
    }

    /// Returns the value assigned to a literal
    pub fn value_of(&self, literal: Literal) -> Option<bool> {
        match self.search.bcp.assignment.literal_value(literal) {
            AssignedValue::True => Some(true),
            AssignedValue::False => Some(false),
            AssignedValue::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_formula() {
        let file = "../test_formulas/or.sat";
        let mut solver = Solver::from_cnf(CNF::from_file_str(file)).without_dlis();
        assert_eq!(solver.solve(), file.contains(".sat"));
    }

    #[test]
    fn test_all_formulas() {
        for entry in fs::read_dir(PathBuf::from("../test_formulas")).unwrap() {
            let file = entry.unwrap();
            dbg!(file.file_name());
            let mut solver = Solver::from_cnf(CNF::from_file(file.path())).without_dlis();
            let sat = solver.solve();
            dbg!(sat);
            assert_eq!(sat, file.file_name().to_str().unwrap().contains(".sat"));
        }
    }

    #[test]
    fn test_history_mode() {
        let cnf = CNF::from_dimacs("1 2 3 0\n-1 2 0\n-2 0\n-1 -2 0\n-3 -4 5 6 0\n-3 4 0\n");
        let mut solver = Solver::from_cnf(cnf).without_dlis();
        let hist = solver.solve_with_history().0;
        dbg!(hist);
    }

    #[test]
    fn test_history_mode_file() {
        let file = "../test_formulas/php_3_2.unsat";
        let mut solver = Solver::from_cnf(CNF::from_file_str(file)).without_dlis();
        let hist = solver.solve_with_history().0;
        dbg!(hist);
    }
}
