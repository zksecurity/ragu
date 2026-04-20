use lean_extraction_macros::extract_gadget;
use ragu_pasta::{EpAffine, Fp};
use ragu_primitives::{Element, Point};

extract_gadget!(
    PointAddIncompleteInstance,
    Fp,
    |p1: Point<_, EpAffine>, p2: Point<_, EpAffine>, nonzero: Element<_>, dr| {
        let p3 = p1.add_incomplete(dr, &p2, Some(&mut nonzero))?;
        Ok((p3, nonzero))
    }
);
