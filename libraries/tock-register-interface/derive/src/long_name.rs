// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{Input, Operation, Read, Write};
use proc_macro2::TokenStream;
use quote::quote;

/// Generates implementations of the `ReadLongName` and `WriteLongName` traits.
pub fn long_name_impls(input: &Input) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let impls = input.registers.iter().flat_map(|register| {
        let name = &register.name;
        register.operations.iter().filter_map(move |op| match op {
            Operation::Read(Read { long_name, .. }) => Some(quote! {
                impl<A: #accessor> #tock_registers::ReadLongName for registers::#name<A> {
                    type LongName = #long_name;
                }
            }),
            Operation::Write(Write { long_name, .. }) => Some(quote! {
                impl<A: #accessor> #tock_registers::WriteLongName for registers::#name<A> {
                    type LongName = #long_name;
                }
            }),
            _ => None,
        })
    });
    quote! { #(#impls)* }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn complex() {
        assert_tokens_eq(
            long_name_impls(&parse_quote! {tock_registers; Foo<A: BusWidth, B: Config> {
                0x0                => a: Accessor::A { Read },
                Accessor::A::WIDTH => b: Accessor::B::BReg { Read, Write },
                _                  => c: u8[3] { Write },
            }}),
            quote! {
                impl<A: Foo> tock_registers::ReadLongName for registers::a<A> {
                    type LongName = ();
                }
                impl<A: Foo> tock_registers::ReadLongName for registers::b<A> {
                    type LongName = ();
                }
                impl<A: Foo> tock_registers::WriteLongName for registers::b<A> {
                    type LongName = ();
                }
                impl<A: Foo> tock_registers::WriteLongName for registers::c<A> {
                    type LongName = ();
                }
            },
        )
    }
}
