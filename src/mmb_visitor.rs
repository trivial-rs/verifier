use crate::statement_iter::{Statement, StatementIter};
use mmb_parser::{ProofStream, UnifyStream, Visitor};
use trivial_kernel::opcode;
use trivial_kernel::Table_;

pub struct UnifyCommands {
    data: Vec<opcode::Command<opcode::Unify>>,
    start_offset: usize,
}

impl UnifyStream for UnifyCommands {
    fn push(&mut self, value: opcode::Command<opcode::Unify>) {
        self.data.push(value);
    }

    fn done(&self) -> (usize, usize) {
        (self.start_offset, self.data.len())
    }
}

pub struct ProofCommands {
    data: Vec<opcode::Command<opcode::Proof>>,
    start_offset: usize,
}

impl ProofStream for ProofCommands {
    fn push(&mut self, value: opcode::Command<opcode::Proof>) {
        self.data.push(value);
    }

    fn done(&self) -> (usize, usize) {
        (self.start_offset, self.data.len())
    }
}

use trivial_kernel::verifier::{Sort_, Term_, Theorem_, Type_};

pub struct MmbVisitor<'a> {
    binders: Vec<Type_>,
    slices: Vec<&'a [u8]>,
    statements: Vec<Statement>,
    uni_streams: UnifyCommands,
    proof_stream: ProofCommands,

    sorts: Vec<Sort_>,
    terms: Vec<Term_>,
    theorems: Vec<Theorem_>,
}

impl<'a> MmbVisitor<'a> {
    pub fn new() -> MmbVisitor<'a> {
        MmbVisitor {
            binders: Vec::with_capacity(1024 * 1024),
            slices: Vec::with_capacity(1024 * 1024),
            statements: Vec::with_capacity(1024 * 1024),
            uni_streams: UnifyCommands {
                data: Vec::with_capacity(1024 * 1024),
                start_offset: 0,
            },
            proof_stream: ProofCommands {
                data: Vec::with_capacity(1024 * 1024),
                start_offset: 0,
            },
            sorts: Vec::new(),
            terms: Vec::new(),
            theorems: Vec::new(),
        }
    }

    pub fn into_table(self) -> (trivial_kernel::Table_, StatementIter) {
        (
            Table_ {
                sorts: self.sorts,
                theorems: self.theorems,
                terms: self.terms,
                unify: self.uni_streams.data,
                binders: self.binders,
            },
            StatementIter::new(self.statements, self.proof_stream.data),
        )
    }
}

impl<'a> Visitor<'a> for MmbVisitor<'a> {
    type Binder = Type_;
    type Sort = Sort_;
    type Statement = opcode::Statement;
    type Proof = ProofCommands;
    type Unify = UnifyCommands;

    fn parse_sort(&mut self, sort: Self::Sort) {
        self.sorts.push(sort);
    }

    fn parse_statement(
        &mut self,
        statement: Self::Statement,
        _offset: usize,
        slice: &'a [u8],
        proof: Option<(usize, usize)>,
    ) {
        self.slices.push(slice);

        self.statements.push(Statement {
            code: statement,
            proof,
        });
    }

    fn start_unify_stream(&mut self) -> &mut UnifyCommands {
        self.uni_streams.start_offset = self.uni_streams.data.len();
        &mut self.uni_streams
    }

    fn start_proof_stream(&mut self) -> &mut ProofCommands {
        self.proof_stream.start_offset = self.proof_stream.data.len();
        &mut self.proof_stream
    }

    fn try_reserve_binder_slice(&mut self, nr: usize) -> Option<(&mut [Type_], usize)> {
        let len = self.binders.len();
        let new_len = len + nr;
        self.binders.resize(new_len, From::from(0));

        if let Some(slice) = self.binders.get_mut(len..) {
            Some((slice, len))
        } else {
            None
        }
    }

    fn parse_term(
        &mut self,
        sort: u8,
        binders: (usize, usize),
        ret_type: Self::Binder,
        unify: &'a [u8],
        unify_indices: (usize, usize),
    ) {
        let term = Term_ {
            sort,
            binders: binders.0..binders.1,
            ret_type,
            unify_commands: unify_indices.0..unify_indices.1,
        };

        self.terms.push(term);
        self.slices.push(unify);
    }

    fn parse_theorem(
        &mut self,
        binders: (usize, usize),
        unify: &'a [u8],
        unify_indices: (usize, usize),
    ) {
        let theorem = Theorem_ {
            binders: binders.0..binders.1,
            unify_commands: unify_indices.0..unify_indices.1,
        };

        self.theorems.push(theorem);
        self.slices.push(unify);
    }
}
