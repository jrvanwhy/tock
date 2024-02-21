// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

extern crate proc_macro;

mod accessor;
mod array_register;
mod long_name;
mod mmio;
mod parsing;
mod peripheral;
mod register_impls;
mod registers;

#[cfg(test)]
mod test_util;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Attribute, Expr, Ident, LitInt, Path, Type, TypeParam, Visibility};

use accessor::accessor;
use array_register::array_register_impls;
use long_name::long_name_impls;
use mmio::mmio;
use peripheral::peripheral_struct;
use register_impls::register_impls;
use registers::registers;

/// Please see `tock_registers::peripheral!` for documentation.
#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    let accessor = accessor(&input);
    let array_register_impls = array_register_impls(&input);
    let long_name_impls = long_name_impls(&input);
    let mmio = mmio(&input);
    let peripheral = peripheral_struct(&input);
    let registers = registers(&input);
    let register_impls = register_impls(&input);
    quote! {
        #accessor
        #array_register_impls
        #long_name_impls
        #mmio
        #peripheral
        #registers
        #register_impls
    }
    .into()
}

/// Parsed version of the input to peripheral!.
#[cfg_attr(test, derive(Debug, PartialEq))]
struct Input {
    pub tock_registers: Path,
    pub allow_bus_adapter: bool,
    pub visibility: Visibility,
    pub accessor: Ident,
    // We currently only support type generics (not lifetime and const
    // generics). That can change in the future if we have a use case.
    pub generics: Vec<TypeParam>,
    pub registers: Vec<Register>,
}

/// An individual register within the peripheral.
#[cfg_attr(test, derive(Debug, PartialEq))]
struct Register {
    pub comments: Vec<Attribute>,
    pub offset: Option<Expr>, // None if the offset is inferred (_)
    pub name: Ident,
    pub data_type: DataType,
    pub operations: Vec<Operation>,
}

/// Type of a register.
#[cfg_attr(test, derive(Debug, PartialEq))]
enum DataType {
    Array { value_type: Type, len: LitInt },
    Scalar { value_type: Type },
}

impl DataType {
    pub fn value_type(&self) -> &Type {
        match self {
            DataType::Array { value_type, .. } => value_type,
            DataType::Scalar { value_type } => value_type,
        }
    }
}

// Note: Currently, we don't support specifying long names on unsafe register
// operations. That can change in the future, when we have a better
// understanding of the use cases of unsafe registers.

#[cfg_attr(test, derive(Debug))]
struct Read {
    pub long_name: Type,
    #[allow(dead_code)] // TODO: Remove
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
struct Write {
    pub long_name: Type,
    #[allow(dead_code)] // TODO: Remove
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
struct UnsafeRead {
    #[allow(dead_code)] // TODO: Remove
    pub span: Span,
}

#[cfg_attr(test, derive(Debug))]
struct UnsafeWrite {
    #[allow(dead_code)] // TODO: Remove
    pub span: Span,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum Operation {
    Read(Read),
    Write(Write),
    UnsafeRead(UnsafeRead),
    UnsafeWrite(UnsafeWrite),
}

// PartialEq definitions for the operations. PartialEq ignores the span field,
// to facilitate the parsing tests.

#[cfg(test)]
impl PartialEq for Read {
    fn eq(&self, other: &Read) -> bool {
        self.long_name == other.long_name
    }
}

#[cfg(test)]
impl PartialEq for Write {
    fn eq(&self, other: &Write) -> bool {
        self.long_name == other.long_name
    }
}

#[cfg(test)]
impl PartialEq for UnsafeRead {
    fn eq(&self, _: &UnsafeRead) -> bool {
        true
    }
}

#[cfg(test)]
impl PartialEq for UnsafeWrite {
    fn eq(&self, _: &UnsafeWrite) -> bool {
        true
    }
}
