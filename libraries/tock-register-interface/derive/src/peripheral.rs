// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

//! Generates the Peripheral struct.

use crate::Input;
use proc_macro2::TokenStream;
use quote::quote;

pub fn peripheral_struct(input: &Input) -> TokenStream {
    let visibility = &input.visibility;
    let accessor = &input.accessor;
    let register_accessors = input.registers.iter().map(|register| {
        let comments = &register.comments;
        let name = &register.name;
        quote! {
            #(#comments)*
            pub fn #name(&self) -> registers::#name::<A> {
                registers::#name::new(self.accessor.clone())
            }
        }
    });
    quote! {
        #[derive(Clone)]
        #visibility struct Peripheral<A: #accessor> {
            accessor: A,
        }

        impl<A: #accessor> Peripheral<A> {
            pub fn new(accessor: A) -> Self {
                Self { accessor }
            }

            #(#register_accessors)*
        }
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
            peripheral_struct(&parse_quote! {tock_registers; Foo<R: LiteXRegisters> {
                0x0 => a: u32 {},
            }}),
            quote! {
                #[derive(Clone)]
                struct Peripheral<A: Foo> {
                    accessor: A,
                }

                impl<A: Foo> Peripheral<A> {
                    pub fn new(accessor: A) -> Self {
                        Self { accessor }
                    }

                    pub fn a(&self) -> registers::a::<A> {
                        registers::a::new(self.accessor.clone())
                    }
                }
            },
        );
    }

    #[test]
    fn no_generics() {
        assert_tokens_eq(
            peripheral_struct(&parse_quote! {tock_registers; Foo {
                /// Comment 1.
                0x0 => a: u32 {},
                /// Comment 2.
                /// Comment 3.
                0x4 => b: u8[3] {},
            }}),
            quote! {
                #[derive(Clone)]
                struct Peripheral<A: Foo> {
                    accessor: A,
                }

                impl<A: Foo> Peripheral<A> {
                    pub fn new(accessor: A) -> Self {
                        Self { accessor }
                    }

                    /// Comment 1.
                    pub fn a(&self) -> registers::a::<A> {
                        registers::a::new(self.accessor.clone())
                    }

                    /// Comment 2.
                    /// Comment 3.
                    pub fn b(&self) -> registers::b::<A> {
                        registers::b::new(self.accessor.clone())
                    }
                }
            },
        );
    }
}
