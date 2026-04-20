use ff::Field;
use ragu_core::drivers::Driver;
use ragu_pasta::{EpAffine, EqAffine, Fp, Fq};
use ragu_primitives::Point;

use lean_extraction_macros::extract_gadget;

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
