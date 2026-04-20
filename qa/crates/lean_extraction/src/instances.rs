//! Extraction instances for each gadget we formalize on the Lean side.
//!
//! Each [`extract_gadget!`] invocation produces a `pub struct <Name>` and a
//! [`CircuitInstance`](crate::instance::CircuitInstance) impl that
//! symbolically runs the gadget method through the
//! [`ExtractionDriver`](crate::driver::ExtractionDriver) and emits the
//! corresponding autogen Lean module.

use ff::Field;
use lean_extraction_macros::extract_gadget;
use ragu_core::drivers::Driver;
use ragu_pasta::{EpAffine, EqAffine, Fp, Fq};
use ragu_primitives::{Element, Point};

use crate::driver::ExtractionDriver;

extract_gadget!(
    PointAllocInstanceFp,
    Fp,
    |dr| Point::<_, EpAffine>::alloc(dr, ExtractionDriver::<Fp>::just(|| Fp::ZERO))
);

extract_gadget!(
    PointAllocInstanceFq,
    Fq,
    |dr| Point::<_, EqAffine>::alloc(dr, ExtractionDriver::<Fq>::just(|| Fq::ZERO))
);

extract_gadget!(
    PointDoubleInstance,
    Fp,
    |p: Point<_, EpAffine>, dr| p.double(dr)
);

extract_gadget!(
    PointNegateInstance,
    Fp,
    |p: Point<_, EpAffine>, dr| Ok(p.negate(dr))
);

extract_gadget!(
    PointAddIncompleteInstance,
    Fp,
    |p1: Point<_, EpAffine>, p2: Point<_, EpAffine>, nonzero: Element<_>, dr| {
        let p3 = p1.add_incomplete(dr, &p2, Some(&mut nonzero))?;
        Ok((p3, nonzero))
    }
);
