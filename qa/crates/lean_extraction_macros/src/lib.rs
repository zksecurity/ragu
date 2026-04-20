//! Procedural macros used only by the `lean_extraction` binary.
//!
//! The headline export is [`extract_gadget!`], which collapses a circuit
//! instance declaration to a single invocation that wraps a closure over the
//! typed input gadgets.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ExprClosure, Ident, Pat, PatType, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

/// Emit a `CircuitInstance` implementation from a closure over the gadget's
/// typed inputs.
///
/// ## Usage
///
/// ```ignore
/// extract_gadget!(
///     PointDoubleInstance,
///     Fp,
///     |p: Point<_, EpAffine>, dr| p.double(dr)
/// );
/// ```
///
/// The closure's typed parameters are allocated as input gadgets via
/// `ExtractionDriver::alloc_input`. The last closure parameter is the
/// driver and is passed through untouched (use whatever name you like;
/// `dr` is conventional). The closure body must return
/// `ragu_core::Result<T>` where `T: Gadget` (or a tuple of gadgets); the
/// macro flattens it via `Gadget::to_wires`.
///
/// Expands to:
///
/// ```ignore
/// pub struct PointDoubleInstance;
///
/// impl CircuitInstance for PointDoubleInstance {
///     type Field = Fp;
///
///     fn circuit(dr: &mut ExtractionDriver<Fp>) -> Result<Vec<Expr<Fp>>> {
///         let mut p: Point<_, EpAffine> = dr.alloc_input()?;
///         let __output = ({ p.double(dr) })?;
///         Gadget::to_wires(&__output)
///     }
/// }
/// ```
#[proc_macro]
pub fn extract_gadget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExtractGadgetInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

struct ExtractGadgetInput {
    name: Ident,
    field: Type,
    closure: ExprClosure,
}

impl Parse for ExtractGadgetInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let field: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        // Parse the remainder as an expression; require it to be a closure.
        let expr: Expr = input.parse()?;
        let closure = match expr {
            Expr::Closure(c) => c,
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected a closure as the third argument",
                ));
            }
        };
        Ok(ExtractGadgetInput {
            name,
            field,
            closure,
        })
    }
}

fn expand(input: ExtractGadgetInput) -> syn::Result<TokenStream2> {
    let ExtractGadgetInput {
        name,
        field,
        closure,
    } = input;
    let body = &closure.body;

    let params: Vec<&Pat> = closure.inputs.iter().collect();
    let (driver_pat, input_params) = params.split_last().ok_or_else(|| {
        syn::Error::new(
            closure.span(),
            "closure must take at least a driver parameter",
        )
    })?;

    let driver_ident: &Ident = match driver_pat {
        Pat::Ident(pat_ident) if pat_ident.subpat.is_none() => &pat_ident.ident,
        _ => {
            return Err(syn::Error::new(
                driver_pat.span(),
                "the last closure parameter must be a plain driver ident (e.g. `dr`)",
            ));
        }
    };

    let input_lets: Vec<TokenStream2> = input_params
        .iter()
        .map(|p| match p {
            Pat::Type(PatType { pat, ty, .. }) => {
                Ok(quote! { let mut #pat: #ty = #driver_ident.alloc_input()?; })
            }
            _ => Err(syn::Error::new(
                p.span(),
                "input parameters must be typed (e.g. `p: Point<_, EpAffine>`)",
            )),
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        pub struct #name;

        impl crate::instance::CircuitInstance for #name {
            type Field = #field;

            fn circuit(
                #driver_ident: &mut crate::driver::ExtractionDriver<#field>,
            ) -> ::ragu_core::Result<::std::vec::Vec<crate::expr::Expr<#field>>> {
                #(#input_lets)*
                let __output = ({ #body })?;
                ::ragu_core::gadgets::Gadget::to_wires(&__output)
            }
        }
    })
}
