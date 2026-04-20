use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::{Element, Point};

use crate::driver::ExtractionDriver;
use crate::expr::Expr;
use crate::instance::{CircuitInstance, WireCollector};

pub struct PointAddIncompleteInstance;

impl CircuitInstance for PointAddIncompleteInstance {
    type Field = Fp;

    fn circuit(dr: &mut ExtractionDriver<Fp>) -> ragu_core::Result<Vec<Expr<Fp>>> {
        let p1: Point<_, EpAffine> = dr.alloc_input()?;
        let p2: Point<_, EpAffine> = dr.alloc_input()?;
        let mut nonzero: Element<_> = dr.alloc_input()?;

        let p3 = p1.add_incomplete(dr, &p2, Some(&mut nonzero))?;

        let mut out = WireCollector::collect_from(&p3)?;
        out.append(&mut WireCollector::collect_from(&nonzero)?);
        Ok(out)
    }
}
