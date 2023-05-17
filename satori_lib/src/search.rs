mod dlis;

use crate::analyze::{analyze, ConflictAnalysis};
use crate::bcp::{propagate, trail, BcpContext};
use crate::search::dlis::DLIS;

/// outer data structures for CDCL search
#[derive(Default)]
pub struct SearchContext {
    pub bcp: BcpContext,
    pub conflict_analysis: ConflictAnalysis,
    pub dlis: DLIS,
}

pub fn search(ctx: &mut SearchContext) -> Option<bool> {
    if ctx.bcp.is_unsat {
        return Some(false);
    }

    let bcp_result = propagate(&mut ctx.bcp);

    match bcp_result {
        Err(conflict) => {
            // conflict without assumptions -> UNSAT
            if ctx.bcp.trail.current_decision_level() == trail::TOP_DECISION_LEVEL {
                ctx.bcp.is_unsat = true;
                return Some(false);
            }
            // or we learn an asserting clause, and backtrack
            analyze(conflict, &mut ctx.conflict_analysis, &mut ctx.bcp);
        }
        Ok(_) => {
            if let Some(literal) = ctx.dlis.decide() {
                // no conflict but not all variables are assigned -> heuristic decision
                trail::decide_and_assign(&mut ctx.bcp, literal);
            } else {
                // no conflict and all variables assigned -> SAT
                return Some(true);
            }
        }
    }

    None
}