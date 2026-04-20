//! `from_wires → to_wires` roundtrip tests for primitive gadgets.
//!
//! Uses a test-local [`SymbolicDriver`] whose `Wire = u32` so we can feed
//! distinct wire indices in and assert the exact same sequence comes back.
//! `MaybeKind = Empty` keeps `from_wires_gadget` usable.
//!
//! Companion to `ragu_core`'s roundtrip tests; those cover foreign-type and
//! hand-written impls, while this file covers derive-generated
//! ([`Element`]) and cross-crate primitive wrappers ([`FixedVec`],
//! [`Demoted`]).

use core::marker::PhantomData;

use ff::Field;
use ragu_arithmetic::Coeff;
use ragu_core::{
    Result,
    drivers::{Driver, DriverTypes},
    gadgets::Gadget,
    maybe::Empty,
};
use ragu_pasta::Fp;
use ragu_primitives::{
    Element, GadgetExt,
    promotion::Demoted,
    vec::{ConstLen, FixedVec},
};

/// Tiny test driver whose wires are bare indices. All circuit-synthesis
/// methods panic; only `from_wires_gadget` / `map_gadget` (via `map`) are
/// exercised.
struct SymbolicDriver<F: Field>(PhantomData<F>);

impl<F: Field> DriverTypes for SymbolicDriver<F> {
    type ImplField = F;
    type ImplWire = u32;
    type MaybeKind = Empty;
    type LCadd = ();
    type LCenforce = ();
    type Extra = ();

    fn gate(
        &mut self,
        _: impl Fn() -> Result<(Coeff<F>, Coeff<F>, Coeff<F>)>,
    ) -> Result<(u32, u32, u32, ())> {
        unreachable!("SymbolicDriver is for roundtrip tests only")
    }

    fn assign_extra(&mut self, _: (), _: impl Fn() -> Result<Coeff<F>>) -> Result<u32> {
        unreachable!("SymbolicDriver is for roundtrip tests only")
    }
}

impl<'dr, F: Field> Driver<'dr> for SymbolicDriver<F> {
    type F = F;
    type Wire = u32;
    const ONE: u32 = 0;

    fn constant(&mut self, _: Coeff<F>) -> u32 {
        unreachable!("SymbolicDriver is for roundtrip tests only")
    }

    fn add(&mut self, _: impl Fn(Self::LCadd) -> Self::LCadd) -> u32 {
        unreachable!("SymbolicDriver is for roundtrip tests only")
    }

    fn enforce_zero(&mut self, _: impl Fn(Self::LCenforce) -> Self::LCenforce) -> Result<()> {
        unreachable!("SymbolicDriver is for roundtrip tests only")
    }
}

type S = SymbolicDriver<Fp>;

fn check_roundtrip<'dr, G: Gadget<'dr, S>>(wires: Vec<u32>) {
    let mut iter = wires.iter().copied();
    let g = G::from_wires(&mut iter).expect("from_wires failed");
    assert!(iter.next().is_none(), "from_wires did not consume all wires");
    let recovered = g.to_wires().expect("to_wires failed");
    assert_eq!(recovered, wires);
}

#[test]
fn element_roundtrip() {
    // `Element` is derive-generated: one `#[ragu(wire)]` field and one
    // `#[ragu(value)]` field. Exercises the macro's derive output.
    check_roundtrip::<'static, Element<'static, S>>(vec![77]);
}

#[test]
fn element_promote_to_wires() {
    // Also exercise the opposite direction: construct an Element manually
    // via the public `promote` API and check `to_wires` picks up the wire.
    let e: Element<'static, S> = Element::promote(42, Empty);
    assert_eq!(e.to_wires().unwrap(), vec![42]);
}

#[test]
fn fixedvec_element_roundtrip() {
    check_roundtrip::<'static, FixedVec<Element<'static, S>, ConstLen<4>>>(vec![1, 2, 3, 4]);
}

#[test]
fn demoted_element_roundtrip() {
    // Demoted<_, Element<_>> — the demotion gadget wrapping an Element.
    check_roundtrip::<'static, Demoted<'static, S, Element<'static, S>>>(vec![99]);
}

#[test]
fn demoted_fixedvec_roundtrip() {
    // Stack Demoted on top of FixedVec<Element> to exercise the recursion
    // through both wrappers.
    check_roundtrip::<
        'static,
        Demoted<'static, S, FixedVec<Element<'static, S>, ConstLen<3>>>,
    >(vec![10, 20, 30]);
}
