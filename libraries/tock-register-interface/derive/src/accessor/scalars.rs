// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::accessor::Parts;
use crate::Operation;
use quote::{format_ident, quote_spanned};
use syn::{Ident, Path, Type};

/// Generates the accessor method for an operation on a scalar register.
pub fn scalar_method(tock_registers: &Path, name: &Ident, op: &Operation, value: &Type) -> Parts {
    let name_offset = format_ident!("{}_offset", name);
    let name_read = format_ident!("{}_read", name);
    let name_write = format_ident!("{}_write", name);
    match op {
        Operation::Read(_) => {
            let doc = format!("Read the {} register.", name);
            Parts {
                definition: quote_spanned! {name.span()=>
                    #[doc = #doc]
                    fn #name_read(&self) -> #value {
                        #tock_registers::reexport::unimplemented!()
                    }
                },
                deref_impl: quote_spanned! {name.span()=>
                    fn #name_read(&self) -> #value {
                        self.deref().#name_read()
                    }
                },
                mmio_impl: quote_spanned! {name.span()=>
                    fn #name_read(&self) -> #value {
                        // TODO: Safety doc
                        unsafe {
                            self.pointer.read::<_, _, Self::#name_offset>(self.bus_adapter)
                        }
                    }
                },
            }
        }
        Operation::Write(_) => {
            let doc = format!("Write the {} register.", name);
            Parts {
                definition: quote_spanned! {name.span()=>
                    #[doc = #doc]
                    fn #name_write(&self, value: #value) {
                        #tock_registers::reexport::unimplemented!();
                    }
                },
                deref_impl: quote_spanned! {name.span()=>
                    fn #name_write(&self, value: #value) {
                        self.deref().#name_write()
                    }
                },
                mmio_impl: quote_spanned! {name.span()=>
                    fn #name_write(&self, value: #value) {
                        // TODO: Safety doc
                        unsafe {
                            self.pointer.write::<_, _, Self::#name_offset>(self.bus_adapter, value)
                        }
                    }
                },
            }
        }
        Operation::UnsafeRead(_) => {
            let doc = [
                format!("Read the {} register.", name),
                "# Safety".into(),
                format!(
                    "Reading the {} register has hardware-specific safety requirements.",
                    name
                ),
            ];
            Parts {
                definition: quote_spanned! {name.span()=>
                    #(#[doc = #doc])*
                    unsafe fn #name_read(&self) -> #value {
                        #tock_registers::reexport::unimplemented!()
                    }
                },
                deref_impl: quote_spanned! {name.span()=>
                    unsafe fn #name_read(&self) -> #value {
                        // TODO: Safety doc
                        unsafe {
                            self.deref().#name_read()
                        }
                    }
                },
                mmio_impl: quote_spanned! {name.span()=>
                    unsafe fn #name_read(&self) -> #value {
                        // TODO: Safety doc
                        unsafe {
                            self.pointer.read::<_, _, Self::#name_offset>(self.bus_adapter)
                        }
                    }
                },
            }
        }
        Operation::UnsafeWrite(_) => {
            let doc = [
                format!("Write the {} register.", name),
                "# Safety".into(),
                format!(
                    "Writing the {} register has hardware-specific safety requirements.",
                    name
                ),
            ];
            Parts {
                definition: quote_spanned! {name.span()=>
                    #(#[doc = #doc])*
                    unsafe fn #name_write(&self, value: #value) {
                        #tock_registers::reexport::unimplemented!();
                    }
                },
                deref_impl: quote_spanned! {name.span()=>
                    unsafe fn #name_write(&self, value: #value) {
                        // TODO: Safety doc
                        unsafe {
                            self.deref().#name_write(value)
                        }
                    }
                },
                mmio_impl: quote_spanned! {name.span()=>
                    unsafe fn #name_write(&self, value: #value) {
                        // TODO: Safety doc
                        unsafe {
                            self.pointer.write::<_, _, Self::#name_offset>(self.bus_adapter, value)
                        }
                    }
                },
            }
        }
    }
}

// TODO: Test
