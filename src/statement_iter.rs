use trivial_kernel::opcode;
use trivial_kernel::verifier::stream::{self, statement::StatementStream};

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

impl StatementIter {
    pub fn new(data: Vec<Statement>, proofs: Vec<opcode::Command<opcode::Proof>>) -> StatementIter {
        StatementIter {
            data: data.into_iter(),
            proofs: Some(proofs.into_iter()),
            ps: None,
        }
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

#[derive(Debug)]
pub struct ProofIter {
    proofs: std::vec::IntoIter<opcode::Command<opcode::Proof>>,
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
