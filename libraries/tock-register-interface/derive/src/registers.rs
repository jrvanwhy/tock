// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{DataType, Input, Operation, Register};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Generates the registers module.
pub fn registers(input: &Input) -> TokenStream {
    let visibility = &input.visibility;
    let contents = input.registers.iter().map(|r| register(input, r));
    quote! {
        #visibility mod registers {
            #![allow(non_camel_case_types)]
            #(#contents)*
        }
    }
}

/// Generates a particular register struct and its in-module impls.
fn register(input: &Input, register: &Register) -> TokenStream {
    use DataType::{Array, Scalar};
    use Operation::{Read, UnsafeRead, UnsafeWrite, Write};
    let comments = &register.comments;
    let name = &register.name;
    let accessor = &input.accessor;
    let fn_new_comment = new_comment();
    let op_impls = register
        .operations
        .iter()
        .map(|op| match (&register.data_type, op) {
            (Array { .. }, Read(_)) => array_read(input, name),
            (Array { .. }, Write(_)) => array_write(input, name),
            (Array { .. }, UnsafeRead(_)) => array_unsafe_read(input, name),
            (Array { .. }, UnsafeWrite(_)) => array_unsafe_write(input, name),
            (Scalar { .. }, Read(_)) => read(input, name),
            (Scalar { .. }, Write(_)) => write(input, name),
            (Scalar { .. }, UnsafeRead(_)) => unsafe_read(input, name),
            (Scalar { .. }, UnsafeWrite(_)) => unsafe_write(input, name),
        });
    quote! {
        #(#comments)*
        #[derive(Clone)]
        pub struct #name<A: super::#accessor> { accessor: A }

        impl<A: super::#accessor> #name<A> {
            #fn_new_comment
            pub fn new(accessor: A) -> Self {
                Self { accessor }
            }
        }

        #(#op_impls)*
    }
}

/// Returns the doc comment for `new()`.
fn new_comment() -> TokenStream {
    quote! {
        /// Constructs this register. Normally, you would use the Peripheral
        /// struct to create register references instead of calling `new`
        /// directly.
    }
}

/// Generates an ArrayRead implementation.
fn array_read(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_read_unchecked = format_ident!("{}_read_unchecked", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::ArrayRead for #name<A> {
            unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                // Safety: ArrayRead::read_unchecked requires index < LEN, which
                // is the only precondition #name_read_unchecked has.
                unsafe {
                    self.accessor.#name_read_unchecked(index)
                }
            }
        }
    }
}

/// Generates an ArrayWrite implementation.
fn array_write(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_write_unchecked = format_ident!("{}_write_unchecked", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::ArrayWrite for #name<A> {
            unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                // Safety: ArrayWrite::write_unchecked requires index < LEN,
                // which is the only precondition #name_write_unchecked has.
                unsafe {
                    self.accessor.#name_write_unchecked(index, value);
                }
            }
        }
    }
}

/// Generates an ArrayUnsafeRead implementation.
fn array_unsafe_read(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_read_unchecked = format_ident!("{}_read_unchecked", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::ArrayUnsafeRead for #name<A> {
            unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                // Safety: ArrayUnsafeRead::read_unchecked requires the caller
                // to comply with the hardware-specific safety requirement, and
                // requires index < LEN. Those are the two safety requirements
                // of #name_read_unchecked.
                unsafe {
                    self.accessor.#name_read_unchecked(index)
                }
            }
        }
    }
}

/// Generates an ArrayUnsafeWrite implementation.
fn array_unsafe_write(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_write_unchecked = format_ident!("{}_write_unchecked", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::ArrayUnsafeWrite for #name<A> {
            unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                // Safety: ArrayUnsafeWrite::write_unchecked requires the caller
                // to comply with the hardware-specific safety requirements, and
                // requires index < LEN. Those are the two safety requirements
                // of #name_write_unchecked.
                unsafe {
                    self.accessor.#name_write_unchecked(index, value);
                }
            }
        }
    }
}

/// Generates a Read implementation.
fn read(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_read = format_ident!("{}_read", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::Read for #name<A> {
            fn read(&self) -> Self::Value {
                self.accessor.#name_read()
            }
        }
    }
}

/// Generates a Write implementation.
fn write(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_write = format_ident!("{}_write", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::Write for #name<A> {
            fn write(&self, value: Self::Value) {
                self.accessor.#name_write(value);
            }
        }
    }
}

/// Generates an UnsafeRead implementation.
fn unsafe_read(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_read = format_ident!("{}_read", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::UnsafeRead for #name<A> {
            unsafe fn read(&self) -> Self::Value {
                // Safety: UnsafeRead::readrequires the caller to comply with
                // the hardware-specific safety requirements, which are the only
                // safety requirements of #name_read.
                unsafe {
                    self.accessor.#name_read()
                }
            }
        }
    }
}

/// Generates an UnsafeWrite implementation.
fn unsafe_write(input: &Input, name: &Ident) -> TokenStream {
    let accessor = &input.accessor;
    let tock_registers = &input.tock_registers;
    let name_write = format_ident!("{}_write", name);
    quote! {
        impl<A: super::#accessor> #tock_registers::UnsafeWrite for #name<A> {
            unsafe fn write(&self, value: Self::Value) {
                // Safety: UnsafeWrite::write requires the caller to comply with
                // the hardware-specific safety requirements, which are the only
                // safety requirements of #name_write.
                unsafe {
                    self.accessor.#name_write(value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn doc_comments() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                0x0 => a: u32 {},
                /// Comment 1.
                /// Comment 2.
                0x4 => b: u8[4] {},
                /// Comment 3.
                0x8 => c: u16 {},
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self { Self { accessor } }
                    }

                    /// Comment 1.
                    /// Comment 2.
                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new (accessor: A) -> Self { Self { accessor } }
                    }

                    /// Comment 3.
                    #[derive(Clone)]
                    pub struct c<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> c<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self { Self { accessor } }
                    }
                }
            },
        );
    }

    // All the other test cases use 0 or 1 operation per register, so this one
    // uses multiple to verify that works.
    #[test]
    fn multiple_ops() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                _ => a: u8 { Read, UnsafeWrite },
                _ => b: u8[2] { UnsafeRead, Write },
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::Read for a<A> {
                        fn read(&self) -> Self::Value {
                            self.accessor.a_read()
                        }
                    }
                    impl<A: super::Foo> tock_registers::UnsafeWrite for a<A> {
                        unsafe fn write(&self, value: Self::Value) {
                            unsafe { self.accessor.a_write(value); }
                        }
                    }

                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayUnsafeRead for b<A> {
                        unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                            unsafe { self.accessor.b_read_unchecked(index) }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayWrite for b<A> {
                        unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                            unsafe { self.accessor.b_write_unchecked(index, value); }
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn read() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                _ => a: u8 { Read },
                _ => b: u8(Ctrl) { Read },
                _ => c: u8[2] { Read },
                _ => d: u8[2] { Read(Ctrl) },
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::Read for a<A> {
                        fn read(&self) -> Self::Value {
                            self.accessor.a_read()
                        }
                    }

                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::Read for b<A> {
                        fn read(&self) -> Self::Value {
                            self.accessor.b_read()
                        }
                    }

                    #[derive(Clone)]
                    pub struct c<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> c<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayRead for c<A> {
                        unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                            unsafe { self.accessor.c_read_unchecked(index) }
                        }
                    }

                    #[derive(Clone)]
                    pub struct d<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> d<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayRead for d<A> {
                        unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                            unsafe { self.accessor.d_read_unchecked(index) }
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn write() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                _ => a: u8 { Write },
                _ => b: u8(Ctrl) { Write },
                _ => c: u8[2] { Write },
                _ => d: u8[2] { Write(Ctrl) },
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::Write for a<A> {
                        fn write(&self, value: Self::Value) {
                            self.accessor.a_write(value);
                        }
                    }

                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::Write for b<A> {
                        fn write(&self, value: Self::Value) {
                            self.accessor.b_write(value);
                        }
                    }

                    #[derive(Clone)]
                    pub struct c<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> c<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayWrite for c<A> {
                        unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                            unsafe { self.accessor.c_write_unchecked(index, value); }
                        }
                    }

                    #[derive(Clone)]
                    pub struct d<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> d<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayWrite for d<A> {
                        unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                            unsafe { self.accessor.d_write_unchecked(index, value); }
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn unsafe_read() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                _ => a: u8 { UnsafeRead },
                _ => b: u8(Ctrl) { UnsafeRead },
                _ => c: u8[2] { UnsafeRead },
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::UnsafeRead for a<A> {
                        unsafe fn read(&self) -> Self::Value {
                            unsafe { self.accessor.a_read() }
                        }
                    }

                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::UnsafeRead for b<A> {
                        unsafe fn read(&self) -> Self::Value {
                            unsafe { self.accessor.b_read() }
                        }
                    }

                    #[derive(Clone)]
                    pub struct c<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> c<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayUnsafeRead for c<A> {
                        unsafe fn read_unchecked(&self, index: usize) -> Self::Value {
                            unsafe { self.accessor.c_read_unchecked(index) }
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn unsafe_write() {
        let fn_new_comment = new_comment();
        assert_tokens_eq(
            registers(&parse_quote! {tock_registers; Foo {
                _ => a: u8 { UnsafeWrite },
                _ => b: u8(Ctrl) { UnsafeWrite },
                _ => c: u8[2] { UnsafeWrite },
            }}),
            quote! {
                mod registers {
                    #![allow(non_camel_case_types)]

                    #[derive(Clone)]
                    pub struct a<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> a<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::UnsafeWrite for a<A> {
                        unsafe fn write(&self, value: Self::Value) {
                            unsafe { self.accessor.a_write(value); }
                        }
                    }

                    #[derive(Clone)]
                    pub struct b<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> b<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::UnsafeWrite for b<A> {
                        unsafe fn write(&self, value: Self::Value) {
                            unsafe { self.accessor.b_write(value); }
                        }
                    }

                    #[derive(Clone)]
                    pub struct c<A: super::Foo> { accessor: A }
                    impl<A: super::Foo> c<A> {
                        #fn_new_comment
                        pub fn new(accessor: A) -> Self {
                            Self { accessor }
                        }
                    }
                    impl<A: super::Foo> tock_registers::ArrayUnsafeWrite for c<A> {
                        unsafe fn write_unchecked(&self, index: usize, value: Self::Value) {
                            unsafe { self.accessor.c_write_unchecked(index, value); }
                        }
                    }
                }
            },
        );
    }
}
