use crate::assignment::AssignedValue;
use crate::cnf::CNF;
use crate::literal::Literal;
use crate::resize::Resize;
use crate::search::{search, SearchContext};
use std::io;
use std::path::Path;

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

    /// Setup and start logging
    pub fn open_event_log(&mut self, path: &Path, log_bcp: bool) -> io::Result<()> {
        let heuristic = if self.search.use_dlis {
            "dlis"
        } else {
            "first-unassigned"
        };

        let variables = self.variable_count;
        let bcp = &mut self.search.bcp;

        bcp.events.open(path, log_bcp, heuristic)?;
        bcp.events
            .init(variables, &bcp.binary_clauses, &bcp.long_clauses);
        bcp.events.replay_trail(&bcp.trail);

        Ok(())
    }

    pub fn close_event_log(&mut self) -> io::Result<()> {
        self.search.bcp.events.finish()
    }

    /// Check satisfiability of the formula
    pub fn solve(&mut self) -> bool {
        let sat = loop {
            if let Some(result) = search(&mut self.search) {
                break result;
            }
        };

        self.search.bcp.events.result(sat, &self.search.bcp.trail);

        sat
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
}
