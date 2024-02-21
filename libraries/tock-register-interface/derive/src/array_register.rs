// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{DataType, Input};
use proc_macro2::TokenStream;
use quote::quote;

/// Generates implementations of the `ArrayRegister` trait (and also
/// `IntoIterator`).
pub fn array_register_impls(input: &Input) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let impls = input.registers.iter().filter_map(|register| {
        let DataType::Array { len, .. } = &register.data_type else {
            return None;
        };
        let name = &register.name;
        Some(quote! {
            impl<A: #accessor> #tock_registers::ArrayRegister for registers::#name<A> {
                const LEN: usize = #len;
            }
            impl<A: #accessor> #tock_registers::reexport::IntoIterator for registers::#name<A> {
                type Item = #tock_registers::ArrayElement<Self>;
                type IntoIter = #tock_registers::ArrayIter<Self>;
                fn into_iter(self) -> Self::IntoIter {
                    #tock_registers::ArrayIter::new(self)
                }
            }
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
    fn mixed() {
        assert_tokens_eq(
            array_register_impls(&parse_quote! {tock_registers; Foo {
                0x0 => a: u8[2] {},
                _   => b: u8 {},
                _   => c: u8[3] {},
            }}),
            quote! {
                impl<A: Foo> tock_registers::ArrayRegister for registers::a<A> {
                    const LEN: usize = 2;
                }
                impl<A: Foo> tock_registers::reexport::IntoIterator for registers::a<A> {
                    type Item = tock_registers::ArrayElement<Self>;
                    type IntoIter = tock_registers::ArrayIter<Self>;
                    fn into_iter(self) -> Self::IntoIter {
                        tock_registers::ArrayIter::new(self)
                    }
                }

                impl<A: Foo> tock_registers::ArrayRegister for registers::c<A> {
                    const LEN: usize = 3;
                }
                impl<A: Foo> tock_registers::reexport::IntoIterator for registers::c<A> {
                    type Item = tock_registers::ArrayElement<Self>;
                    type IntoIter = tock_registers::ArrayIter<Self>;
                    fn into_iter(self) -> Self::IntoIter {
                        tock_registers::ArrayIter::new(self)
                    }
                }
            },
        )
    }
}
