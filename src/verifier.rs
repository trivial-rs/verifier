use crate::kernel::{
    stream::proof, Context, KResult, State, Stepper, Store_, Table, Table_, Term, Theorem, Var_,
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

    pub fn verify_unify(&self) {
        let mut i = 0;

        let f = |x: u32| self.table.get_term(x).unwrap().get_binders().len() as u32;

        while let Some(term) = self.table.get_term(i) {
            let unify = term.get_command_stream();
            let unify = self.table.get_unify_commands(unify).unwrap();

            let proof =
                trivial_compiler::unify_to_proof(term.get_binders().len() as u32, unify.iter(), f);

            let binders = term.get_binders();
            let binders = self.table.get_binders(binders).unwrap();

            let mut dummy_context = Context::<Store_>::default();

            dummy_context
                .allocate_binders(&self.table, self.state.get_current_sort(), binders)
                .unwrap();

            let mut stepper = proof::Stepper::new(false, self.state, proof.iter().cloned());

            stepper.run(&mut dummy_context, &self.table).unwrap();

            i += 1;
        }

        while let Some(thm) = self.table.get_theorem(i) {
            let unify = thm.get_unify_commands();
            let unify = self.table.get_unify_commands(unify).unwrap();

            let proof =
                trivial_compiler::unify_to_proof(thm.get_binders().len() as u32, unify.iter(), f);

            let binders = thm.get_binders();
            let binders = self.table.get_binders(binders).unwrap();

            let mut dummy_context = Context::<Store_>::default();

            dummy_context
                .allocate_binders(&self.table, self.state.get_current_sort(), binders)
                .unwrap();

            let mut stepper = proof::Stepper::new(false, self.state, proof.iter().cloned());

            stepper.run(&mut dummy_context, &self.table).unwrap();

            i += 1;
        }
    }

    pub fn seek_term(&mut self, idx: usize) {
        let stream = self.stepper.get_stream_mut();

        let idx = *stream.term_indices.get(idx).unwrap();

        stream.seek_to(idx, &mut self.state);
    }

    pub fn seek_theorem(&mut self, idx: usize) {
        let stream = self.stepper.get_stream_mut();

        let idx = *stream.theorem_indices.get(idx).unwrap();

        stream.seek_to(idx, &mut self.state);
    }

    pub fn seek(&mut self, idx: usize) {
        let stream = self.stepper.get_stream_mut();

        stream.seek_to(idx, &mut self.state);
    }

    pub fn create_theorem_application(&self, id: u32) -> Option<(Context<Store_>, usize, usize)> {
        let f = |x: u32| self.table.get_term(x).unwrap().get_binders().len() as u32;

        if let Some(thm) = self.table.get_theorem(id) {
            let unify = thm.get_unify_commands();
            let unify = self.table.get_unify_commands(unify).unwrap();

            let binders = thm.get_binders();
            let nr_args = binders.len();

            let binders = self.table.get_binders(binders).unwrap();

            let mut dummy_context = Context::default();

            dummy_context
                .allocate_binders(&self.table, self.state.get_current_sort(), binders)
                .unwrap();

            let proof =
                trivial_compiler::unify_to_proof(thm.get_binders().len() as u32, unify.iter(), f);

            let mut stepper = proof::Stepper::new(false, self.state, proof.iter().cloned());

            stepper.run(&mut dummy_context, &self.table).unwrap();

            let nr_hyps = dummy_context.get_hyp_stack().len();

            Some((dummy_context, nr_args, nr_hyps))
        } else {
            None
        }
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
