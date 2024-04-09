// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use quote::quote;
use syn::{parse_macro_input, Attribute, Ident, LitInt, Path, Type, Visibility};

mod parsing;

#[proc_macro]
pub fn peripheral(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _input = parse_macro_input!(input as Input);
    quote! {}.into()
}

struct Input {
    pub allow_bus_adapter: bool,
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub fields: Vec<Field>,
    pub name: Ident,
    pub real_name: Ident,
    pub tock_registers: Path,
    pub visibility: Visibility,
}

struct Field {
    pub cfgs: Vec<Attribute>,
    pub comments: Vec<Attribute>,
    pub data_type: Type,
    pub offset: Option<LitInt>,
    pub register: Option<Register>,
}

struct Register {
    pub name: Ident,
    pub long_names: Type,
    pub read: Access,
    pub write: Access,
}

enum LongNames {
    Single(Type),
    Aliased(Aliased),
}

struct Aliased {
    pub read: Type,
    pub write: Type,
}

#[derive(PartialEq)]
enum Access {
    NoAccess,
    Unsafe(Ident),
    Safe(Ident),
}
