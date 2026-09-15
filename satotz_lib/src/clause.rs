use crate::literal::Literal;

/// Type wrapper for readability. The index of the clause in [bcp::long_clauses].
pub type ClauseIndex = usize;
pub type ClauseId = u32;

/// Contains metadata for a clause
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClauseMeta {
    pub id: ClauseId,
}

/// Representation of one long clause (3+ literals) in the propagation datastructure [bcp::long_clauses]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clause {
    header: ClauseMeta,
    literals: Box<[Literal]>,
}

impl Clause {
    pub fn from_lit_vec(literals: Vec<Literal>) -> Clause {
        Clause {
            header: ClauseMeta::default(),
            literals: literals.into_boxed_slice(),
        }
    }

    pub fn from_literals(literals: &[Literal]) -> Clause {
        Self::from_lit_vec(literals.to_vec())
    }

    pub fn header(&self) -> &ClauseMeta {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut ClauseMeta {
        &mut self.header
    }

    pub fn literals(&self) -> &[Literal] {
        &self.literals
    }

    pub fn literals_mut(&mut self) -> &mut [Literal] {
        &mut self.literals
    }
}
