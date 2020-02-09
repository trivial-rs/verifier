use crate::kernel::{
    context::PackedPtr, stream::proof, Context, KResult, State, Stepper, Store_, Table, Table_,
    Term, Theorem, Var_,
};
use crate::mmb_visitor::MmbVisitor;
use crate::statement_iter::StatementOwned;
use mmb_parser::Mmb;

use crate::kernel::stream::statement::Action;

pub struct Verifier {
    pub table: Table_,
    pub context: Context<Store_>,
    pub state: State,
    stepper: Stepper<StatementOwned, Var_>,
}

impl Verifier {
    pub fn new(data: &[u8]) -> Option<Verifier> {
        let data = Mmb::from(data)?;

        let mut visitor = MmbVisitor::new();
        data.visit(&mut visitor).ok()?;

        let (table, stream) = visitor.into_table_owned();

        let stepper = Stepper::new(stream);

        Some(Verifier {
            table,
            stepper,
            context: Context::default(),
            state: State::default(),
        })
    }

    pub fn verify_unify(&self) -> KResult {
        let mut i = 0;

        let f = |x: u32| self.table.get_term(x).unwrap().get_binders().len() as u32;

        while let Some(term) = self.table.get_term(i) {
            let unify = term.get_command_stream();
            let unify = self
                .table
                .get_unify_commands(unify)
                .ok_or(crate::kernel::error::Kind::InvalidUnifyCommandIndex)?;

            let proof =
                trivial_compiler::unify_to_proof(term.get_binders().len() as u32, unify.iter(), f);

            let binders = term.get_binders();
            let binders = self
                .table
                .get_binders(binders)
                .ok_or(crate::kernel::error::Kind::InvalidBinderIndices)?;

            let mut dummy_context = Context::<Store_>::default();

            dummy_context.allocate_binders(&self.table, self.state.get_current_sort(), binders)?;

            let mut stepper = proof::Stepper::new(false, self.state, proof.iter().cloned());

            stepper.run(&mut dummy_context, &self.table)?;

            i += 1;
        }

        while let Some(thm) = self.table.get_theorem(i) {
            let unify = thm.get_unify_commands();
            let unify = self
                .table
                .get_unify_commands(unify)
                .ok_or(crate::kernel::error::Kind::InvalidUnifyCommandIndex)?;

            let proof =
                trivial_compiler::unify_to_proof(thm.get_binders().len() as u32, unify.iter(), f);

            let binders = thm.get_binders();
            let binders = self
                .table
                .get_binders(binders)
                .ok_or(crate::kernel::error::Kind::InvalidBinderIndices)?;

            let mut dummy_context = Context::<Store_>::default();

            dummy_context.allocate_binders(&self.table, self.state.get_current_sort(), binders)?;

            let mut stepper = proof::Stepper::new(false, self.state, proof.iter().cloned());

            stepper.run(&mut dummy_context, &self.table)?;

            i += 1;
        }

        Ok(())
    }

    pub fn seek_term(&mut self, idx: usize) -> bool {
        let stream = self.stepper.get_stream_mut();

        if let Some(idx) = stream.term_indices.get(idx).copied() {
            self.state = stream.seek_to(idx);
            true
        } else {
            false
        }
    }

    pub fn seek_theorem(&mut self, idx: usize) -> bool {
        let stream = self.stepper.get_stream_mut();

        if let Some(idx) = stream.theorem_indices.get(idx).copied() {
            self.state = stream.seek_to(idx);
            true
        } else {
            false
        }
    }

    pub fn seek(&mut self, idx: usize) {
        let stream = self.stepper.get_stream_mut();

        self.state = stream.seek_to(idx);
    }

    pub fn create_theorem_application<'a>(
        &self,
        id: u32,
        context: &'a mut Context<Store_>,
    ) -> KResult<(&'a [PackedPtr], &'a [PackedPtr], PackedPtr)> {
        let f = |x: u32| self.table.get_term(x).unwrap().get_binders().len() as u32;

        let thm = self
            .table
            .get_theorem(id)
            .ok_or(crate::kernel::error::Kind::InvalidTheorem)?;

        let unify = thm.get_unify_commands();
        let unify = self
            .table
            .get_unify_commands(unify)
            .ok_or(crate::kernel::error::Kind::InvalidUnifyCommandIndex)?;

        let binders = thm.get_binders();
        let nr_args = binders.len();

        let binders = self
            .table
            .get_binders(binders)
            .ok_or(crate::kernel::error::Kind::InvalidBinderIndices)?;

        context.clear_except_store();

        let state = State::from_table(&self.table);

        context.allocate_binders(&self.table, state.get_current_sort(), binders)?;

        let proof =
            trivial_compiler::unify_to_proof(thm.get_binders().len() as u32, unify.iter(), f);

        let mut stepper = proof::Stepper::new(false, state, proof.iter().cloned());

        stepper.run(context, &self.table)?;

        let args = context
            .get_proof_heap()
            .as_slice()
            .get(..nr_args)
            .ok_or(crate::kernel::error::Kind::InvalidHeapIndex)?;
        let res = context
            .get_proof_stack()
            .peek()
            .ok_or(crate::kernel::error::Kind::ProofStackUnderflow)?;

        Ok((args, context.get_hyp_stack().as_slice(), *res))
    }

    pub fn step<F: FnMut(Action, &Self)>(&mut self, f: &mut F) -> KResult<Option<()>> {
        let x = self
            .stepper
            .step(&mut self.context, &mut self.state, &self.table)?;

        if let Some(x) = x {
            f(x, self);

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub fn run<F: FnMut(Action, &Self)>(&mut self, f: &mut F) -> KResult<()> {
        while self.step(f)?.is_some() {}

        Ok(())
    }

    pub fn step_statement<F: FnMut(Action, &Self)>(&mut self, f: &mut F) -> KResult<Option<()>> {
        let x = self
            .stepper
            .step(&mut self.context, &mut self.state, &self.table)?;

        if let Some(x) = x {
            f(x, self);

            if !self.stepper.is_state_normal() {
                Ok(Some(()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn run_statement<F: FnMut(Action, &Self)>(&mut self, f: &mut F) -> KResult<()> {
        while self.step_statement(f)?.is_some() {}

        Ok(())
    }
}
