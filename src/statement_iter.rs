use crate::kernel::opcode;
use crate::kernel::stream::{self, statement::StatementStream};
use crate::kernel::State;

#[derive(Debug)]
pub struct Statement {
    pub code: opcode::Statement,
    pub proof: Option<(usize, usize)>,
}

#[derive(Debug)]
pub struct StatementIter {
    data: std::vec::IntoIter<Statement>,
    proofs: Option<std::vec::IntoIter<opcode::Command<opcode::Proof>>>,
    ps: Option<(usize, usize)>,
}

#[derive(Debug)]
pub struct StatementOwned {
    data: Vec<Statement>,

    pub sort_indices: Vec<usize>,
    pub axiom_indices: Vec<usize>,
    pub term_indices: Vec<usize>,
    pub theorem_indices: Vec<usize>,

    idx: usize,
    proofs: Option<Vec<opcode::Command<opcode::Proof>>>,
    ps: Option<(usize, usize)>,
}

impl StatementIter {
    pub fn new(data: Vec<Statement>, proofs: Vec<opcode::Command<opcode::Proof>>) -> StatementIter {
        StatementIter {
            data: data.into_iter(),
            proofs: Some(proofs.into_iter()),
            ps: None,
        }
    }
}

impl StatementOwned {
    pub fn new(
        data: Vec<Statement>,
        proofs: Vec<opcode::Command<opcode::Proof>>,
        sort_indices: Vec<usize>,
        axiom_indices: Vec<usize>,
        term_indices: Vec<usize>,
        theorem_indices: Vec<usize>,
    ) -> StatementOwned {
        StatementOwned {
            data,
            sort_indices,
            axiom_indices,
            term_indices,
            theorem_indices,
            idx: 0,
            proofs: Some(proofs),
            ps: None,
        }
    }

    pub fn seek_to(&mut self, idx: usize) -> State {
        let mut state = State::default();

        for i in self.data.iter().take(idx) {
            use opcode::Statement;
            match i.code {
                Statement::End => break,
                Statement::Sort => state.increment_current_sort(),
                Statement::TermDef => state.increment_current_term(),
                Statement::LocalDef => state.increment_current_term(),
                Statement::LocalTerm => state.increment_current_term(),
                Statement::Axiom => state.increment_current_theorem(),
                Statement::Thm => state.increment_current_theorem(),
            }
        }

        self.idx = idx;

        state
    }
}

impl Iterator for StatementIter {
    type Item = stream::statement::Opcode;

    fn next(&mut self) -> Option<stream::statement::Opcode> {
        self.ps = None;

        if let Some(statement) = self.data.next() {
            use opcode::Statement;
            match statement.code {
                Statement::End => Some(stream::statement::Opcode::End),
                Statement::Sort => Some(stream::statement::Opcode::Sort),
                Statement::TermDef => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::LocalDef => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::LocalTerm => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::Axiom => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::Axiom)
                }
                Statement::Thm => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::Thm)
                }
            }
        } else {
            None
        }
    }
}

impl Iterator for StatementOwned {
    type Item = stream::statement::Opcode;

    fn next(&mut self) -> Option<stream::statement::Opcode> {
        self.ps = None;

        if let Some(statement) = self.data.get(self.idx) {
            self.idx += 1;
            use opcode::Statement;
            match statement.code {
                Statement::End => Some(stream::statement::Opcode::End),
                Statement::Sort => Some(stream::statement::Opcode::Sort),
                Statement::TermDef => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::LocalDef => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::LocalTerm => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::TermDef)
                }
                Statement::Axiom => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::Axiom)
                }
                Statement::Thm => {
                    self.ps = statement.proof;
                    Some(stream::statement::Opcode::Thm)
                }
            }
        } else {
            None
        }
    }
}

impl StatementStream for StatementIter {
    type ProofStream = ProofIter;

    fn take_proof_stream(&mut self) -> Self::ProofStream {
        let len = self.ps.unwrap_or((0, 0));
        ProofIter {
            proofs: self.proofs.take().unwrap(),
            max_len: (len.1 - len.0),
        }
    }

    fn put_proof_stream(&mut self, proofs: Self::ProofStream) {
        self.proofs = Some(proofs.proofs);
    }
}

impl StatementStream for StatementOwned {
    type ProofStream = ProofOwned;

    fn take_proof_stream(&mut self) -> Self::ProofStream {
        let len = self.ps.unwrap_or((0, 0));
        ProofOwned {
            proofs: self.proofs.take().unwrap(),
            idx: len.0,
            max_len: (len.1 - len.0),
        }
    }

    fn put_proof_stream(&mut self, proofs: Self::ProofStream) {
        self.proofs = Some(proofs.proofs);
    }
}

#[derive(Debug)]
pub struct ProofIter {
    proofs: std::vec::IntoIter<opcode::Command<opcode::Proof>>,
    max_len: usize,
}

#[derive(Debug)]
pub struct ProofOwned {
    proofs: Vec<opcode::Command<opcode::Proof>>,
    idx: usize,
    max_len: usize,
}

impl Iterator for ProofIter {
    type Item = opcode::Command<opcode::Proof>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_len == 0 {
            None
        } else {
            self.max_len -= 1;
            self.proofs.next()
        }
    }
}

impl Iterator for ProofOwned {
    type Item = opcode::Command<opcode::Proof>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_len == 0 {
            None
        } else {
            self.max_len -= 1;
            let ret = self.proofs.get(self.idx);
            self.idx += 1;

            ret.copied()
        }
    }
}
