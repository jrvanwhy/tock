// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::accessor::Parts;
use quote::{format_ident, quote_spanned};
use syn::{Ident, LitInt, Path, Type};

/// Accessor definitions and implementations for UnsafeWrite operations on array
/// registers.
pub fn array_unsafe_write(
    tock_registers: &Path,
    name: &Ident,
    value: &Type,
    len: &LitInt,
) -> Parts {
    let assert_msg = format!(
        "index out of bounds while writing {}: the len is {} but the index is {{}}",
        name, len
    );
    let doc = [
        format!("Write the {} register.", name),
        format!("Returns `None` if `index` is >= {}.", len),
        "# Safety".into(),
        format!(
            "Writing {} has hardware-specific safety requirements.",
            name
        ),
    ];
    let doc_unchecked = [
        format!("Write the {} register without bounds checking.", name),
        "Most users of tock-registers should not need to call or implement this function.".into(),
        "# Safety".into(),
        format!("`index` must be less than {}.", len),
        format!(
            "Writing {} has hardware-specific safety requirements.",
            name
        ),
    ];
    let expect_msg = format!(
        "{}_write called with in-bounds index but returned None",
        name
    );
    let name_offset = format_ident!("{}_offset", name);
    let name_write = format_ident!("{}_write", name);
    let name_write_unchecked = format_ident!("{}_write_unchecked", name);
    Parts {
        definition: quote_spanned! {name.span()=>
            #(#[doc = #doc])*
            unsafe fn #name_write(&self, index: usize, value: #value) -> #tock_registers::reexport::Result<(), #tock_registers::OutOfBounds> {
                #tock_registers::reexport::unimplemented!()
            }
            #(#[doc = #doc_unchecked])*
            #[doc(hidden)]
            unsafe fn #name_write_unchecked(&self, index: usize, value: #value) {
                #tock_registers::reexport::assert!(index < #len, #assert_msg, index);
                self.#name_write(indexi, value).expect(#expect_msg)
            }
        },
        deref_impl: quote_spanned! {name.span()=>
            unsafe fn #name_write(&self, index: usize, value: #value) -> #tock_registers::reexport::Result<(), #tock_registers::OutOfBounds> {
                // Safety: Implementing Foo::name_write using itself, just
                // forwarding through the arguments.
                unsafe {
                    self.deref().#name_write(index, value)
                }
            }
            unsafe fn #name_write_unchecked(&self, index: usize, value: #value) {
                // Safety: Implementing Foo::name_write_unchecked using itself,
                // just forwarding through the arguments.
                unsafe {
                    self.deref().#name_write_unchecked(index, value)
                }
            }
        },
        mmio_impl: quote_spanned! {name.span()=>
            unsafe fn #name_write(&self, index: usize) -> #tock_registers::reexport::Option<#value> {
                // TODO: Safety comment
                unsafe {
                    self.pointer.array_write::<_, _, Self::#name_offset, #len>(self.bus_adapter, index, value)
                }
            }
            unsafe fn #name_write_unchecked(&self, index: usize) -> #value {
                // TODO: Safety comment
                unsafe {
                    self.pointer.array_write_unchecked::<_, _, Self::#name_offset>(self.bus_adapter, index, value)
                }
            }
        },
    }
}
