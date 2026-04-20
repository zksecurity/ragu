use lean_extraction_macros::extract_gadget;
use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::Point;

extract_gadget!(PointNegateInstance, Fp, |p: Point<_, EpAffine>, dr| Ok(
    p.negate(dr)
));
