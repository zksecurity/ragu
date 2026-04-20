use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::Point;

use crate::driver::ExtractionDriver;
use crate::expr::Expr;
use crate::instance::{CircuitInstance, WireCollector};

pub struct PointDoubleInstance;

impl CircuitInstance for PointDoubleInstance {
    type Field = Fp;

    fn circuit(dr: &mut ExtractionDriver<Fp>) -> ragu_core::Result<Vec<Expr<Fp>>> {
        let input: Point<_, EpAffine> = dr.alloc_input()?;
        let doubled = input.double(dr)?;
        WireCollector::collect_from(&doubled)
    }
}
