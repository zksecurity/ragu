use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::Point;

use lean_extraction_macros::extract_gadget;

extract_gadget!(
    PointDoubleInstance,
    Fp,
    |p: Point<_, EpAffine>, dr| p.double(dr)
);
