//! `from_wires → to_wires` roundtrip tests for hand-written gadget impls.
//!
//! Uses a test-local [`SymbolicDriver`] whose `Wire = u32`, so we can feed
//! distinct wire indices in and assert the exact same sequence comes back
//! out. `MaybeKind = Empty` keeps `from_wires_gadget` usable (real drivers
//! hit a `const panic` when it's invoked).
//!
//! Derive-generated gadgets are covered by `ragu_primitives/tests/roundtrip.rs`
//! (where `#[derive(Gadget)]` resolves `ragu_core`'s path via
//! `proc-macro-crate`). This file covers the foreign-type impls (`()`, arrays,
//! tuples, `Box`) plus small hand-written gadgets (`Single`, `Pair`) that
//! stand in for user code implementing `Gadget` / `GadgetKind` by hand.

use alloc::{boxed::Box, vec, vec::Vec};
use core::marker::PhantomData;

use ff::Field;
use ragu_arithmetic::Coeff;
use ragu_pasta::Fp;

use crate::{
    Result,
    convert::WireMap,
    drivers::{Driver, DriverTypes},
    gadgets::{Gadget, GadgetKind},
    maybe::Empty,
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

// --- test-local hand-written gadgets (1-wire and 2-wire) ---

struct Single<'dr, D: Driver<'dr>> {
    w: D::Wire,
    _marker: PhantomData<&'dr ()>,
}

impl<'dr, D: Driver<'dr>> Clone for Single<'dr, D> {
    fn clone(&self) -> Self {
        Single {
            w: self.w.clone(),
            _marker: PhantomData,
        }
    }
}

struct SingleKind;

/// Safety: `Single` only holds a wire and `PhantomData`; `Send` follows
/// from `D::Wire: Send`.
unsafe impl<FieldType: Field> GadgetKind<FieldType> for SingleKind {
    type Rebind<'dr, D: Driver<'dr, F = FieldType>> = Single<'dr, D>;

    fn map_gadget<
        'src,
        'dst,
        WM: WireMap<FieldType, Src: Driver<'src, F = FieldType>, Dst: Driver<'dst, F = FieldType>>,
    >(
        this: &Single<'src, WM::Src>,
        wm: &mut WM,
    ) -> Result<Single<'dst, WM::Dst>> {
        Ok(Single {
            w: wm.convert_wire(&this.w)?,
            _marker: PhantomData,
        })
    }

    fn enforce_equal_gadget<
        'dr,
        D1: Driver<'dr, F = FieldType>,
        D2: Driver<'dr, F = FieldType, Wire = <D1 as Driver<'dr>>::Wire>,
    >(
        dr: &mut D1,
        a: &Single<'dr, D2>,
        b: &Single<'dr, D2>,
    ) -> Result<()> {
        dr.enforce_equal(&a.w, &b.w)
    }

    fn from_wires_gadget<'dr, D: Driver<'dr, F = FieldType>, I: Iterator<Item = D::Wire>>(
        iter: &mut I,
    ) -> Result<Single<'dr, D>> {
        let w = iter
            .next()
            .ok_or(crate::Error::VectorLengthMismatch { expected: 1, actual: 0 })?;
        Ok(Single {
            w,
            _marker: PhantomData,
        })
    }
}

impl<'dr, D: Driver<'dr>> Gadget<'dr, D> for Single<'dr, D> {
    type Kind = SingleKind;
}

struct Pair<'dr, D: Driver<'dr>> {
    a: D::Wire,
    b: D::Wire,
    _marker: PhantomData<&'dr ()>,
}

impl<'dr, D: Driver<'dr>> Clone for Pair<'dr, D> {
    fn clone(&self) -> Self {
        Pair {
            a: self.a.clone(),
            b: self.b.clone(),
            _marker: PhantomData,
        }
    }
}

struct PairKind;

/// Safety: `Pair` only holds wires and `PhantomData`.
unsafe impl<FieldType: Field> GadgetKind<FieldType> for PairKind {
    type Rebind<'dr, D: Driver<'dr, F = FieldType>> = Pair<'dr, D>;

    fn map_gadget<
        'src,
        'dst,
        WM: WireMap<FieldType, Src: Driver<'src, F = FieldType>, Dst: Driver<'dst, F = FieldType>>,
    >(
        this: &Pair<'src, WM::Src>,
        wm: &mut WM,
    ) -> Result<Pair<'dst, WM::Dst>> {
        Ok(Pair {
            a: wm.convert_wire(&this.a)?,
            b: wm.convert_wire(&this.b)?,
            _marker: PhantomData,
        })
    }

    fn enforce_equal_gadget<
        'dr,
        D1: Driver<'dr, F = FieldType>,
        D2: Driver<'dr, F = FieldType, Wire = <D1 as Driver<'dr>>::Wire>,
    >(
        dr: &mut D1,
        a: &Pair<'dr, D2>,
        b: &Pair<'dr, D2>,
    ) -> Result<()> {
        dr.enforce_equal(&a.a, &b.a)?;
        dr.enforce_equal(&a.b, &b.b)
    }

    fn from_wires_gadget<'dr, D: Driver<'dr, F = FieldType>, I: Iterator<Item = D::Wire>>(
        iter: &mut I,
    ) -> Result<Pair<'dr, D>> {
        let a = iter
            .next()
            .ok_or(crate::Error::VectorLengthMismatch { expected: 2, actual: 0 })?;
        let b = iter
            .next()
            .ok_or(crate::Error::VectorLengthMismatch { expected: 2, actual: 1 })?;
        Ok(Pair {
            a,
            b,
            _marker: PhantomData,
        })
    }
}

impl<'dr, D: Driver<'dr>> Gadget<'dr, D> for Pair<'dr, D> {
    type Kind = PairKind;
}

type S = SymbolicDriver<Fp>;

/// Roundtrip helper: `from_wires(w) |> to_wires == w` for any gadget
/// instantiated over [`SymbolicDriver`].
fn check_roundtrip<'dr, G: Gadget<'dr, S>>(wires: Vec<u32>) {
    let mut iter = wires.iter().copied();
    let g = G::from_wires(&mut iter).expect("from_wires failed");
    assert!(iter.next().is_none(), "from_wires did not consume all wires");
    let recovered = g.to_wires().expect("to_wires failed");
    assert_eq!(recovered, wires);
}

#[test]
fn unit_roundtrip() {
    check_roundtrip::<'static, ()>(vec![]);
}

#[test]
fn single_roundtrip() {
    check_roundtrip::<'static, Single<'static, S>>(vec![42]);
}

#[test]
fn pair_roundtrip() {
    check_roundtrip::<'static, Pair<'static, S>>(vec![7, 9]);
}

#[test]
fn array_roundtrip() {
    check_roundtrip::<'static, [Single<'static, S>; 3]>(vec![1, 2, 3]);
}

#[test]
fn tuple_roundtrip() {
    check_roundtrip::<'static, (Single<'static, S>, Pair<'static, S>)>(vec![10, 20, 30]);
}

#[test]
fn box_roundtrip() {
    check_roundtrip::<'static, Box<Pair<'static, S>>>(vec![100, 200]);
}

#[test]
fn nested_roundtrip() {
    // Box<[(Single, Pair); 2]> — exercises every foreign impl together.
    check_roundtrip::<'static, Box<[(Single<'static, S>, Pair<'static, S>); 2]>>(vec![
        1, 2, 3, 11, 22, 33,
    ]);
}
