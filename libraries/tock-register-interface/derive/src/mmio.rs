// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{DataType, Input};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

/// Generates the Mmio* struct. Note that this does not generate the Accessor
/// impl for the struct; that is done by the `accessor` module.
pub fn mmio(input: &Input) -> TokenStream {
    let Generics {
        generic_params,
        type_args,
    } = generics(input);
    let mmio_name = format_ident!("Mmio{}", input.accessor);
    let tock_registers = &input.tock_registers;
    let visibility = &input.visibility;

    let (bus_type, bus_arg, bus_init) = match input.allow_bus_adapter {
        false => (
            quote! { #tock_registers::DirectBus },
            quote! {},
            quote! { bus_adapter: #tock_registers::DirectBus },
        ),
        true => (
            quote! { B },
            quote! { bus_adapter: B },
            quote! { bus_adapter },
        ),
    };
    // The expected offset of the next register.
    let mut expect_offset = Some(quote! {0});
    let offsets = input.registers.iter().map(|register| {
        let name_offset = format_ident!("{}_offset", register.name);
        // If an offset was supplied, use that offset for name_offset and emit
        // an offset test as well. If no offset was supplied, use the expected
        // offset for the impl.
        let offset = match &register.offset {
            None => expect_offset.take().unwrap(),
            Some(offset) => {
                let error_message = format!("Unexpected offset for register {}", register.name);
                quote_spanned! {offset.span()=> {
                    assert!(#offset == #expect_offset, #error_message);
                    #offset
                }}
            }
        };
        // Build the expected offset for the next register.
        let value = register.data_type.value_type();
        let value_size = quote! { <#bus_type as #tock_registers::BusAdapter<#value>>::SIZE };
        let total_size = match &register.data_type {
            DataType::Array { len, .. } => quote! { #len * #value_size },
            DataType::Scalar { .. } => value_size,
        };
        expect_offset = Some(quote! { Self::#name_offset + #total_size });
        quote! {
            const #name_offset: usize = #offset;
        }
    });
    // If allow_bus_adapter is true, then the offset test has to be in new(), as
    // it is dependent on the BusAdapter chosen. If allow_bus_adapter is false,
    // we put it into a `const`, so it fires on a normal `cargo check`.
    let (offset_test_new, offset_test_const) = match input.registers.last() {
        None => (quote! {}, quote! {}),
        Some(register) => {
            let name_offset = format_ident!("{}_offset", register.name);
            match input.allow_bus_adapter {
                false => (
                    quote! {},
                    quote! { const _: usize = #mmio_name::<#tock_registers::DynPointer>::#name_offset; },
                ),
                true => (quote! { let _ = Self::#name_offset; }, quote! {}),
            }
        }
    };

    quote! {
        /// The "real" implementation of the accessor trait, which uses
        /// MMIO memory accesses to control the peripheral.
        // Safety invariant:
        // The `peripheral!` invocation correctly describes the register layout
        // of the MMIO peripheral that `pointer` points to. `bus_adapter` is the
        // correct bus adapter for that MMIO region.
        #[derive(Clone)]
        #visibility struct #mmio_name<#generic_params> {
            bus_adapter: #bus_type,
            pointer: M,
        }

        impl<#generic_params> #mmio_name<#type_args> {
            #![allow(non_upper_case_globals)]
            #(#offsets)*

            /// # Safety
            /// Preconditions:
            /// 1. The `peripheral!` invocation correctly describes the
            ///    register layout of the MMIO peripheral that `pointer`
            ///    points to.
            /// 2. If the peripheral is `#[allow_bus_adapter]`, then
            ///    `bus_adapter` must be the correct bus adapter.
            pub unsafe fn new(pointer: M, #bus_arg) -> Self {
                #offset_test_new
                Self {
                    #bus_init,
                    pointer,
                }
            }
        }

        #offset_test_const
    }
}

pub struct Generics {
    pub generic_params: TokenStream, // M: MmioPointer, B: BusAdapter<...> + ...
    pub type_args: TokenStream,      // M, B
}

/// Returns the generics bounds and type arguments for an `impl` on the Mmio*
/// struct.
pub fn generics(input: &Input) -> Generics {
    let tock_registers = &input.tock_registers;
    if !input.allow_bus_adapter {
        return Generics {
            generic_params: quote! { M: #tock_registers::MmioPointer },
            type_args: quote! { M },
        };
    }
    let bus_bounds = input.registers.iter().map(|register| {
        let value = register.data_type.value_type();
        quote! { #tock_registers::BusAdapter<#value> }
    });
    Generics {
        generic_params: quote! { M: #tock_registers::MmioPointer, B: #(#bus_bounds)+* },
        type_args: quote! { M, B },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn allow_bus_adapter() {
        assert_tokens_eq(
            mmio(&parse_quote! {tock_registers; #[allow_bus_adapter] Foo {
                0x0 => a: u32 {},
                _   => b: u8[3] {},
                0x7 => c: u8 {},
            }}),
            quote! {
                /// The "real" implementation of the accessor trait, which uses
                /// MMIO memory accesses to control the peripheral.
                #[derive(Clone)]
                struct MmioFoo<
                    M: tock_registers::MmioPointer,
                    B: tock_registers::BusAdapter<u32> + tock_registers::BusAdapter<u8>
                        + tock_registers::BusAdapter<u8>
                > {
                    bus_adapter: B,
                    pointer: M,
                }

                impl<
                    M: tock_registers::MmioPointer,
                    B: tock_registers::BusAdapter<u32> + tock_registers::BusAdapter<u8>
                        + tock_registers::BusAdapter<u8>
                > MmioFoo<M, B> {
                    #![allow(non_upper_case_globals)]
                    const a_offset: usize = {
                        assert!(0x0 == 0, "Unexpected offset for register a");
                        0x0
                    };
                    const b_offset: usize = Self::a_offset +
                        <B as tock_registers::BusAdapter<u32>>::SIZE;
                    const c_offset: usize = {
                        assert!(0x7 == Self::b_offset + 3 *
                            <B as tock_registers::BusAdapter<u8>>::SIZE,
                            "Unexpected offset for register c");
                        0x7
                    };

                    /// # Safety
                    /// Preconditions:
                    /// 1. The `peripheral!` invocation correctly describes the
                    ///    register layout of the MMIO peripheral that `pointer`
                    ///    points to.
                    /// 2. If the peripheral is `#[allow_bus_adapter]`, then
                    ///    `bus_adapter` must be the correct bus adapter.
                    pub unsafe fn new(pointer: M, bus_adapter: B) -> Self {
                        let _ = Self::c_offset;
                        Self {
                            bus_adapter,
                            pointer,
                        }
                    }
                }
            },
        )
    }

    #[test]
    fn no_bus_adapter() {
        assert_tokens_eq(
            mmio(&parse_quote! {tock_registers; Foo {
                0x0 => a: u32 {},
                _   => b: u8[3] {},
                0x7 => c: u8 {},
            }}),
            quote! {
                /// The "real" implementation of the accessor trait, which uses
                /// MMIO memory accesses to control the peripheral.
                #[derive(Clone)]
                struct MmioFoo<M: tock_registers::MmioPointer> {
                    bus_adapter: tock_registers::DirectBus,
                    pointer: M,
                }

                impl<M: tock_registers::MmioPointer> MmioFoo<M> {
                    #![allow(non_upper_case_globals)]
                    const a_offset: usize = {
                        assert!(0x0 == 0, "Unexpected offset for register a");
                        0x0
                    };
                    const b_offset: usize = Self::a_offset +
                        <tock_registers::DirectBus as tock_registers::BusAdapter<u32>>::SIZE;
                    const c_offset: usize = {
                        assert!(0x7 == Self::b_offset + 3 *
                            <tock_registers::DirectBus as tock_registers::BusAdapter<u8>>::SIZE,
                            "Unexpected offset for register c");
                        0x7
                    };

                    /// # Safety
                    /// Preconditions:
                    /// 1. The `peripheral!` invocation correctly describes the
                    ///    register layout of the MMIO peripheral that `pointer`
                    ///    points to.
                    /// 2. If the peripheral is `#[allow_bus_adapter]`, then
                    ///    `bus_adapter` must be the correct bus adapter.
                    pub unsafe fn new(pointer: M,) -> Self {
                        Self {
                            bus_adapter: tock_registers::DirectBus,
                            pointer,
                        }
                    }
                }

                const _: usize = MmioFoo::<tock_registers::DynPointer>::c_offset;
            },
        )
    }
}
