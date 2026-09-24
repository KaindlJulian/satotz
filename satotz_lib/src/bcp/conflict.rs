use crate::bcp::BcpContext;
use crate::clause::ClauseIndex;
use crate::literal::Literal;

/// Set of literals that make the formula unsat when conjunctively added to the formula
#[derive(Debug)]
pub enum Conflict {
    BinaryClause([Literal; 2]),
    LongClause(ClauseIndex),
}

impl Conflict {
    pub fn get_literals<'a>(&'a self, context: &'a BcpContext) -> &'a [Literal] {
        match self {
            Conflict::BinaryClause(literals) => literals,
            Conflict::LongClause(clause_index) => context.long_clauses.literals(*clause_index),
        }
    }
}
