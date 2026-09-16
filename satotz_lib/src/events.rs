use crate::bcp::binary_clauses::{BinaryClause, BinaryClauses};
use crate::bcp::long_clauses::LongClauses;
use crate::bcp::trail::{Reason, Step, Trail};
use crate::clause::{ClauseId, ClauseIndex};
use crate::literal::Literal;
use serde::{Serialize, Serializer};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub const PROTOCOL_VERSION: &str = "1";

const BUF_BYTES: usize = 0xfffff; // 1 MiB

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InspectOutcome {
    Satisfied,
    Unit,
    Falsified,
    Unresolved,
}

#[derive(Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum Event<'a> {
    Init {
        protocol_version: &'static str,
        variables: usize,
        clauses: usize,
        variable_ids: VariableIds,
        clause_list: &'a [ClauseEntry],
    },
    Decide {
        literal: Literal,
        level: u32,
        heuristic: &'static str,
    },
    Propagate {
        literal: Literal,
        level: u32,
        reason_clause_id: Option<ClauseId>,
    },
    Conflict {
        clause_id: ClauseId,
        literals: &'a [Literal],
        level: u32,
        trail: Assigned<'a>,
    },
    Learn {
        learned_literals: &'a [Literal],
        clause_id: i64,
        jump_level: u32,
    },
    Backtrack {
        from_level: u32,
        to_level: u32,
        kind: &'static str,
        reason: &'static str,
    },
    Result {
        result: &'static str,
        model: Assigned<'a>,
    },
    Inspect {
        clause_id: ClauseId,
        outcome: InspectOutcome,
        watched: [Literal; 2],
        #[serde(skip_serializing_if = "Option::is_none")]
        next_watched: Option<[Literal; 2]>,
    },
}

#[derive(Serialize)]
struct ClauseEntry {
    id: ClauseId,
    literals: Vec<Literal>,
}

impl Serialize for Literal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_dimacs_integer().serialize(serializer)
    }
}

/// The assigned literals of a slice of trail steps.
struct Assigned<'a>(&'a [Step]);

impl Serialize for Assigned<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|step| step.assigned_literal))
    }
}

/// [1, 2, ..., n] without allocating.
struct VariableIds(usize);

impl Serialize for VariableIds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(1..=self.0)
    }
}

#[derive(Debug)]
pub struct EventLog {
    out: Option<BufWriter<File>>,
    bcp: bool, // inspection event toggle
    heuristic: &'static str,
    error: Option<io::Error>,
}

impl Default for EventLog {
    fn default() -> Self {
        Self {
            out: None,
            bcp: false,
            heuristic: "dlis",
            error: None,
        }
    }
}

impl EventLog {
    pub fn open(&mut self, path: &Path, bcp: bool, heuristic: &'static str) -> io::Result<()> {
        self.out = Some(BufWriter::with_capacity(BUF_BYTES, File::create(path)?));
        self.bcp = bcp;
        self.heuristic = heuristic;
        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if let Some(out) = self.out.as_mut() {
            let flushed = out.flush();
            self.keep_first_error(flushed);
        }
        self.error.take().map_or(Ok(()), Err)
    }

    /// event: init
    pub fn init(&mut self, variables: usize, binary: &BinaryClauses, long: &LongClauses) {
        if self.out.is_none() {
            return;
        }

        let mut clause_list: Vec<ClauseEntry> = vec![];
        for (code, entries) in binary.literal_lookup().iter().enumerate() {
            let literal = Literal::from_code(code);
            for entry in entries {
                clause_list.push(ClauseEntry {
                    id: entry.header.id,
                    literals: vec![literal, entry.other_literal],
                });
            }
        }

        // deduplicate binary clauses
        clause_list.sort_by_key(|c| c.id);
        clause_list.dedup_by_key(|c| c.id);
        clause_list.extend(long.clauses().iter().map(|c| ClauseEntry {
            id: c.header().id,
            literals: c.literals().to_vec(),
        }));
        clause_list.sort_by_key(|c| c.id);

        self.emit(&Event::Init {
            protocol_version: PROTOCOL_VERSION,
            variables,
            clauses: clause_list.len(),
            variable_ids: VariableIds(variables),
            clause_list: &clause_list,
        });
    }

    /// handles root-level propagations
    pub fn replay_trail(&mut self, trail: &Trail) {
        for step in trail.steps() {
            self.propagate(step.assigned_literal, step.decision_level, None);
        }
    }

    pub fn assign(&mut self, step: &Step) {
        match step.reason {
            Reason::SolverDecision => self.decide(step.assigned_literal, step.decision_level),
            Reason::Unit => self.propagate(step.assigned_literal, step.decision_level, None),
            Reason::Binary(_, id) | Reason::Long(_, id) => {
                self.propagate(step.assigned_literal, step.decision_level, Some(id))
            }
        }
    }

    pub fn conflict_binary(
        &mut self,
        clause: &BinaryClause,
        false_literal: Literal,
        trail: &Trail,
    ) {
        self.conflict(
            clause.header.id,
            &[false_literal, clause.other_literal],
            trail,
        );
    }

    /// event: conflict
    pub fn conflict(&mut self, clause_id: ClauseId, literals: &[Literal], trail: &Trail) {
        self.emit(&Event::Conflict {
            clause_id,
            literals,
            level: trail.current_decision_level(),
            trail: Assigned(trail.steps()),
        });
    }

    /// event: learn
    pub fn learn(&mut self, literals: &[Literal], next_clause_id: ClauseId, jump_level: u32) {
        let clause_id = if literals.len() > 1 {
            next_clause_id as i64
        } else {
            -1 // unit clause
        };

        self.emit(&Event::Learn {
            learned_literals: literals,
            clause_id,
            jump_level,
        });
    }

    /// event: backtrack
    pub fn backtrack(&mut self, trail: &Trail, to_level: u32) {
        self.emit(&Event::Backtrack {
            from_level: trail.current_decision_level(),
            to_level,
            kind: "conflict",
            reason: "analyze",
        });
    }

    /// event: result
    pub fn result(&mut self, sat: bool, trail: &Trail) {
        self.emit(&Event::Result {
            result: if sat { "sat" } else { "unsat" },
            model: Assigned(if sat { trail.steps() } else { &[] }),
        });
    }

    /// event: decide
    fn decide(&mut self, literal: Literal, level: u32) {
        self.emit(&Event::Decide {
            literal,
            level,
            heuristic: self.heuristic,
        });
    }

    /// event: propagate
    fn propagate(&mut self, literal: Literal, level: u32, reason: Option<ClauseId>) {
        self.emit(&Event::Propagate {
            literal,
            level,
            reason_clause_id: reason,
        });
    }

    /// todo: not emit(?) long clause whose blocking literal stopped the BCP read
    pub fn inspect_blocked(&mut self, clauses: &LongClauses, index: ClauseIndex) {
        let clause = &clauses.clauses()[index];
        let literals = clause.literals();
        self.inspect(
            clause.header().id,
            InspectOutcome::Satisfied,
            [literals[0], literals[1]],
            None,
        );
    }

    pub fn inspect_long(
        &mut self,
        clause_id: ClauseId,
        watched: [Literal; 2],
        outcome: InspectOutcome,
    ) {
        self.inspect(clause_id, outcome, watched, None);
    }

    pub fn inspect_long_moved(
        &mut self,
        clause_id: ClauseId,
        watched: [Literal; 2],
        next: [Literal; 2],
    ) {
        self.inspect(clause_id, InspectOutcome::Unresolved, watched, Some(next));
    }

    pub fn inspect_binary(
        &mut self,
        clause: &BinaryClause,
        false_literal: Literal,
        outcome: InspectOutcome,
    ) {
        self.inspect(
            clause.header.id,
            outcome,
            [false_literal, clause.other_literal],
            None,
        );
    }

    /// event: inspect
    fn inspect(
        &mut self,
        clause_id: ClauseId,
        outcome: InspectOutcome,
        watched: [Literal; 2],
        next_watched: Option<[Literal; 2]>,
    ) {
        if self.bcp {
            self.emit(&Event::Inspect {
                clause_id,
                outcome,
                watched,
                next_watched,
            });
        }
    }

    /// serializes one event to JSON
    fn emit(&mut self, event: &Event<'_>) {
        if let Some(out) = self.out.as_mut() {
            let written = serde_json::to_writer(&mut *out, event)
                .map_err(io::Error::from)
                .and_then(|()| out.write_all(b"\n"));
            self.keep_first_error(written);
        }
    }

    fn keep_first_error(&mut self, result: io::Result<()>) {
        if let Err(e) = result {
            if self.error.is_none() {
                self.error = Some(e);
            }
        }
    }
}
