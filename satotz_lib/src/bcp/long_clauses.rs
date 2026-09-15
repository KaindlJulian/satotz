use crate::clause::{Clause, ClauseId, ClauseIndex};
use crate::literal::Literal;

/// Holds all long clauses during propagation
#[derive(Default, Debug)]
pub struct LongClauses {
    clauses: Vec<Clause>,
}

impl LongClauses {
    pub fn clauses(&self) -> &Vec<Clause> {
        &self.clauses
    }

    pub fn add_clause(&mut self, literals: &[Literal], id: ClauseId) -> usize {
        let mut clause = Clause::from_literals(literals);
        clause.header_mut().id = id;
        self.clauses.push(clause);
        self.clauses.len() - 1
    }

    pub fn find_clause_mut(&mut self, index: ClauseIndex) -> &mut Clause {
        self.clauses.get_mut(index).expect("no clause found")
    }

    pub fn literals(&self, index: ClauseIndex) -> &[Literal] {
        self.clauses[index].literals()
    }
}
