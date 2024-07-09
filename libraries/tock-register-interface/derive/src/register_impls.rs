// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::Input;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

/// Generates implementations of the `Register` trait.
pub fn register_impls(input: &Input) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let impls = input.registers.iter().map(|register| {
        let name = &register.name;
        let value = register.data_type.value_type();
        quote_spanned! {name.span()=>
            impl<Accessor: #accessor> #tock_registers::Register for registers::#name<Accessor> {
                type Value = #value;
            }
        }
    });
    quote! {
        #(#impls)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn generics() {
        assert_tokens_eq(
            register_impls(&parse_quote! {tock_registers; Foo<A: BusWidth, B: Config> {
                0x0                => a: Accessor::A {},
                Accessor::A::WIDTH => b: Accessor::B::BReg {},
                _                  => c: u8[3] {},
            }}),
            quote! {
                impl<Accessor: Foo> tock_registers::Register for registers::a<Accessor> {
                    type Value = Accessor::A;
                }
                impl<Accessor: Foo> tock_registers::Register for registers::b<Accessor> {
                    type Value = Accessor::B::BReg;
                }
                impl<Accessor: Foo> tock_registers::Register for registers::c<Accessor> {
                    type Value = u8;
                }
            },
        )
    }

    #[test]
    fn no_generics() {
        assert_tokens_eq(
            register_impls(&parse_quote! {tock_registers; Foo {
                0x0 => a: u32 {},
                _   => b: u8[3] {},
                0x7 => c: u8 {},
            }}),
            quote! {
                impl<Accessor: Foo> tock_registers::Register for registers::a<Accessor> {
                    type Value = u32;
                }
                impl<Accessor: Foo> tock_registers::Register for registers::b<Accessor> {
                    type Value = u8;
                }
                impl<Accessor: Foo> tock_registers::Register for registers::c<Accessor> {
                    type Value = u8;
                }
            },
        )
    }
}
