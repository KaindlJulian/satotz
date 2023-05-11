use crate::bcp::long_clauses::{ClauseIndex, LongClauses};
use crate::literal::Literal;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct LiteralWatch {
    /// index of the clause being watched
    pub clause_index: ClauseIndex,
}

/// For every literal, keeps a list of clauses watched by this literal
/// The watched literals will always be the first two, at index 0 and 1
#[derive(Default)]
pub struct Watchlists {
    watches: HashMap<Literal, Vec<LiteralWatch>>,
}

impl Watchlists {
    /// watch a clause with 2 instances of [`LiteralWatch`]
    pub fn watch_clause(&mut self, clause_index: ClauseIndex, lits: [Literal; 2]) {
        for i in 0..2 {
            let watch = LiteralWatch { clause_index };

            match self.watches.entry(lits[i]) {
                Entry::Occupied(mut e) => e.get_mut().push(watch),
                Entry::Vacant(e) => {
                    e.insert(vec![watch]);
                }
            }
        }
    }

    /// Take ownership of a literals watchlist
    pub fn take_watchlist(&self, lit: Literal) -> Vec<LiteralWatch> {
        self.watches.get(&lit).unwrap().drain(..).collect()
    }

    pub fn place_watchlist(&mut self, lit: Literal, watchlist: Vec<LiteralWatch>) {
        self.watches.insert(lit, watchlist);
    }
}

fn build_watchlists(watch: &mut Watchlists, long_clauses: &LongClauses) {
    for (index, clause) in long_clauses.clauses.iter().enumerate() {
        let literals = clause.literals();
        watch.watch_clause(index, [literals[0], literals[1]]);
    }
}
