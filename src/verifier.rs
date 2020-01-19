use crate::mmb_visitor::MmbVisitor;
use crate::statement_iter::StatementIter;
use mmb_parser::Mmb;
use trivial_kernel::{verifier::state::store::Store_, verifier::Type_, State, Stepper, Table_};

pub struct Verifier {
    table: Table_,
    state: State<Store_>,
    stepper: Stepper<StatementIter, Type_>,
}

impl Verifier {
    pub fn new(data: &[u8]) -> Option<Verifier> {
        let data = Mmb::from(data)?;

        let mut visitor = MmbVisitor::new();
        data.visit(&mut visitor).ok()?;

        let (table, stream) = visitor.into_table();

        let stepper = Stepper::new(stream);

        Some(Verifier {
            table,
            stepper,
            state: State::default(),
        })
    }

    pub fn step(&mut self) -> Result<Option<()>, ()> {
        match self.stepper.step(&mut self.state, &self.table) {
            Ok(x) => {
                if x.is_some() {
                    Ok(Some(()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err(()),
        }
    }
}
