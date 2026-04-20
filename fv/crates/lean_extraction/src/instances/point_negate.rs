use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::Point;

use lean_extraction_macros::extract_gadget;

extract_gadget!(
    PointNegateInstance,
    Fp,
    |p: Point<_, EpAffine>, dr| Ok(p.negate(dr))
);
