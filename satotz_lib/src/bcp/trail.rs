use crate::assignment::VariableAssignment;
use crate::bcp::BcpContext;
use crate::clause::ClauseIndex;
use crate::literal::{Literal, Variable};
use crate::resize::Resize;

pub type StepIndex = usize;

pub static TOP_DECISION_LEVEL: u32 = 0;

#[derive(Debug, PartialEq, Clone)]
pub enum Reason {
    /// Decided by the solver
    SolverDecision,
    /// Implied by a unit clause
    Unit,
    /// Implied by a binary clause because the given other literal of the binary clause is false.
    Binary(Literal),
    /// Implied by a long clause because all but its first literal are false.
    Long(ClauseIndex),
}

impl Reason {
    /// Returns the falsified literals that caused the propagation
    pub fn causing_literals<'a>(&'a self, context: &'a BcpContext) -> &'a [Literal] {
        match self {
            Reason::SolverDecision | Reason::Unit => &[],
            Reason::Binary(literal) => std::slice::from_ref(literal),
            Reason::Long(clause_index) => &context.long_clauses.literals(*clause_index)[1..],
        }
    }
}

/// A step in the implication graph
#[derive(Debug, Clone)]
pub struct Step {
    pub assigned_literal: Literal,
    pub decision_level: u32,
    pub reason: Reason,
}

#[derive(Debug, Clone)]
pub struct Trail {
    /// represents the implication graph
    steps: Vec<Step>,
    /// maps a variable to the index of the corresponding entry in `steps`
    step_index_by_var: Vec<StepIndex>,
    propagated: usize,
    decisions: Vec<u32>,
}

impl Resize for Trail {
    fn resize(&mut self, var_count: usize) {
        self.step_index_by_var.resize(var_count, 0);
    }
}

impl Default for Trail {
    fn default() -> Self {
        Trail {
            steps: vec![],
            step_index_by_var: Default::default(),
            propagated: 0,
            decisions: vec![0],
        }
    }
}

impl Trail {
    pub fn step_index(&self, variable: Variable) -> StepIndex {
        self.step_index_by_var[variable.as_index()]
    }

    pub fn increase_propagated(&mut self) {
        self.propagated += 1;
    }

    pub fn next_unpropagated_literal(&self) -> Option<Literal> {
        self.steps
            .get(self.propagated)
            .map(|step| step.assigned_literal)
    }

    pub fn current_decision_level(&self) -> u32 {
        self.decisions.len() as u32 - 1
    }

    pub fn steps(&self) -> &Vec<Step> {
        &self.steps
    }

    /// Returns the step where given variable was assigned
    #[allow(dead_code)] // used for tests
    pub fn get_step_for_variable(&self, var: Variable) -> &Step {
        &self.steps[self.step_index(var)]
    }
}

/// adds given step to the trail, assigning the literal
pub fn assign(values: &mut VariableAssignment, trail: &mut Trail, step: Step) {
    trail.step_index_by_var[step.assigned_literal.variable().as_index()] = trail.steps.len();
    values.assign_true(step.assigned_literal);
    trail.steps.push(step);
}

/// adds a solver decision to the trail, assigning the literal
pub fn decide_and_assign(bcp: &mut BcpContext, literal: Literal) {
    bcp.trail.decisions.push(bcp.trail.steps.len() as u32);
    let step = Step {
        assigned_literal: literal,
        decision_level: bcp.trail.current_decision_level(),
        reason: Reason::SolverDecision,
    };
    assign(&mut bcp.assignment, &mut bcp.trail, step);
}

/// backtracks to given decision level, undoing assignments of a higher level
pub fn backtrack(bcp: &mut BcpContext, target_decision_level: u32) {
    // backtrack target must be lower than current decision level
    assert!(target_decision_level < bcp.trail.current_decision_level());

    // Get the index corresponding to the lowest decision to undo
    let decision_level = target_decision_level as usize;
    let target_trail_len = bcp.trail.decisions[decision_level + 1] as usize;

    // Undo the assignments
    for step in bcp.trail.steps.drain(target_trail_len..) {
        let variable = step.assigned_literal.variable();
        bcp.assignment.assign_unknown(variable);
    }

    // remove from graph
    bcp.trail.decisions.truncate(decision_level + 1);
    bcp.trail.propagated = bcp.trail.propagated.min(target_trail_len);
}
