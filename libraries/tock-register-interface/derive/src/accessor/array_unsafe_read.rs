// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::accessor::Parts;
use quote::{format_ident, quote_spanned};
use syn::{Ident, LitInt, Path, Type};

/// Accessor definitions and implementations for UnsafeRead operations on array
/// registers.
pub fn array_unsafe_read(tock_registers: &Path, name: &Ident, value: &Type, len: &LitInt) -> Parts {
    let assert_msg = format!(
        "index out of bounds while reading {}: the len is {} but the index is {{}}",
        name, len
    );
    let doc = [
        format!("Read the {} register.", name),
        format!("Returns `None` if `index` is >= {}.", len),
        "# Safety".into(),
        format!(
            "Reading {} has hardware-specific safety requirements.",
            name
        ),
    ];
    let doc_unchecked = [
        format!("Read the {} register without bounds checking.", name),
        "Most users of tock-registers should not need to call or implement this function.".into(),
        "# Safety".into(),
        format!("`index` must be less than {}.", len),
        format!(
            "Reading {} has hardware-specific safety requirements.",
            name
        ),
    ];
    let expect_msg = format!(
        "{}_read called with in-bounds index but returned None",
        name
    );
    let name_offset = format_ident!("{}_offset", name);
    let name_read = format_ident!("{}_read", name);
    let name_read_unchecked = format_ident!("{}_read_unchecked", name);
    Parts {
        definition: quote_spanned! {name.span()=>
            #(#[doc = #doc])*
            unsafe fn #name_read(&self, index: usize) -> #tock_registers::reexport::Option<#value> {
                #tock_registers::reexport::unimplemented!()
            }
            #(#[doc = #doc_unchecked])*
            #[doc(hidden)]
            unsafe fn #name_read_unchecked(&self, index: usize) -> #value {
                #tock_registers::reexport::assert!(index < #len, #assert_msg, index);
                self.#name_read(index).expect(#expect_msg)
            }
        },
        deref_impl: quote_spanned! {name.span()=>
            unsafe fn #name_read(&self, index: usize) -> #tock_registers::reexport::Option<#value> {
                // TODO: Safety comment
                unsafe {
                    self.deref().#name_read(index)
                }
            }
            unsafe fn #name_read_unchecked(&self, index: usize) -> #value {
                // Safety: Implementing Foo::name_read_unchecked using itself,
                // just forwarding through the arguments.
                unsafe {
                    self.deref().#name_read_unchecked(index)
                }
            }
        },
        mmio_impl: quote_spanned! {name.span()=>
            unsafe fn #name_read(&self, index: usize) -> #tock_registers::reexport::Option<#value> {
                // Safety: That `OFFSET`, `LEN`, `Value`, and `bus_adapter` are
                // correct is guaranteed by the safety invariant that the
                // peripheral! invocation is correct (and pointer points to it).
                // The hardware-specific safety requirements were complied with
                // by the caller.
                unsafe {
                    self.pointer.array_read::<_, _, Self::#name_offset, #len>(self.bus_adapter, index)
                }
            }
            unsafe fn #name_read_unchecked(&self, index: usize) -> #value {
                // Safety: That `OFFSET`, `LEN`, `Value`, and `bus_adapter` are
                // correct is guaranteed by the safety invariant that the
                // peripheral! invocation is correct (and pointer points to it).
                // The hardware-specific safety requirements were complied with
                // by the caller. The caller guaranteed that `index` is in-bounds.
                unsafe {
                    self.pointer.array_read_unchecked::<_, _, Self::#name_offset>(self.bus_adapter, index)
                }
            }
        },
    }
}
