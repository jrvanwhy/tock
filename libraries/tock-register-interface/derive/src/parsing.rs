// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

//! Parsing logic for the proc macro (mostly `syn::parse::Parse` impls).

use crate::{DataType, Input, Operation, Read, Register, UnsafeRead, UnsafeWrite, Write};
use proc_macro2::Span;
use syn::parse::{self, Parse, ParseStream};
use syn::token::{Bracket, Paren};
use syn::{
    braced, bracketed, parenthesized, parse_quote, Attribute, GenericParam, Generics, Ident, Meta,
    Token, Type,
};

// The input syntax looks like:
//     tock_registers; pub Foo<L: LiteXRegisters> {
//         0x8 => padding: u32 {},
//         0xc => ctrl: u8 { Read, Write },
//     }
impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Input> {
        Ok(Input {
            tock_registers: input.parse()?,
            allow_bus_adapter: {
                input.parse::<Token![;]>()?;
                match input.call(Attribute::parse_outer)?.as_slice() {
                    [] => false,
                    [attr] => match &attr.meta {
                        Meta::Path(path) if path.is_ident("allow_bus_adapter") => true,
                        _ => return Err(input.error("Unknown attribute")),
                    },
                    _ => {
                        return Err(input
                            .error("At most one attribute is allowed on a peripheral definition"))
                    }
                }
            },
            visibility: { input.parse()? },
            accessor: input.parse()?,
            generics: {
                let parsed: Generics = input.parse()?;
                let mut generics = Vec::with_capacity(parsed.params.len());
                for generic in parsed.params {
                    let GenericParam::Type(param) = generic else {
                        return Err(input.error("Generic parameters must be types"));
                    };
                    generics.push(param);
                }
                generics
            },
            registers: {
                let registers;
                braced!(registers in input);
                registers
                    .parse_terminated(Register::parse, Token![,])?
                    .into_iter()
                    .collect()
            },
        })
    }
}

// Checks whether a parsed Attribute represents a doc comment.
fn is_doc_comment(attr: &Attribute) -> bool {
    let Meta::NameValue(ref name_value) = attr.meta else {
        return false;
    };
    let Some(ident) = name_value.path.get_ident() else {
        return false;
    };
    *ident == "doc"
}

// The register input syntax looks like:
//     0x8 => ctrl: u8[4](UartCtrl) { Read, Write }
//     0xc => fifo: u32 { Read(RxByte), Write(TxByte) }
impl Parse for Register {
    fn parse(input: ParseStream) -> parse::Result<Register> {
        Ok(Register {
            comments: {
                let comments = input.call(Attribute::parse_outer)?;
                if !comments.iter().all(is_doc_comment) {
                    return Err(input.error(
                        "Attributes other than doc comments are not supported on registers.",
                    ));
                }
                comments
            },
            offset: {
                match input.peek(Token![_]) {
                    true => {
                        input.parse::<Token![_]>()?;
                        None
                    }
                    false => Some(input.parse()?),
                }
            },
            name: {
                input.parse::<Token![=>]>()?;
                input.parse()?
            },
            data_type: {
                input.parse::<Token![:]>()?;
                input.parse()?
            },
            operations: {
                let shared_long_name = maybe_long_name(input)?;
                let operations;
                braced!(operations in input);
                operations
                    .parse_terminated(OpSpec::parse, Token![,])?
                    .into_iter()
                    .map(|op| {
                        Ok(match op {
                            OpSpec::Read(span, long_name) => Operation::Read(Read {
                                long_name: build_long_name(&shared_long_name, long_name, span)?,
                                span,
                            }),
                            OpSpec::Write(span, long_name) => Operation::Write(Write {
                                long_name: build_long_name(&shared_long_name, long_name, span)?,
                                span,
                            }),
                            OpSpec::UnsafeRead(span) => Operation::UnsafeRead(UnsafeRead { span }),
                            OpSpec::UnsafeWrite(span) => {
                                Operation::UnsafeWrite(UnsafeWrite { span })
                            }
                        })
                    })
                    .collect::<parse::Result<_>>()?
            },
        })
    }
}

// Combines a shared_long_name with an OpSpec's long_name to produce the correct
// long_name for an Operation.
fn build_long_name(shared: &Option<Type>, spec: Option<Type>, span: Span) -> parse::Result<Type> {
    match (shared, spec) {
        (None, None) => Ok(parse_quote! {()}),
        (None, Some(long_name)) => Ok(long_name),
        (Some(long_name), None) => Ok(long_name.clone()),
        (Some(_), Some(_)) => Err(parse::Error::new(
            span,
            "Cannot specify LongName on both the register data type and an operation",
        )),
    }
}

// A data type is written by writing the value type followed, optionally, by an
// array length. For example:
//     `u32` would represent a scalar register.
//     `u8[4]` would represent an array register.
impl Parse for DataType {
    fn parse(input: ParseStream) -> parse::Result<DataType> {
        let value_type = input.parse()?;
        Ok(match input.peek(Bracket) {
            false => DataType::Scalar { value_type },
            true => DataType::Array {
                value_type,
                len: {
                    let len;
                    bracketed!(len in input);
                    len.parse()?
                },
            },
        })
    }
}

// A specification for an operation. Examples:
//     UnsafeRead
//     Read
//     Write(Foo)
// A specification does not necessarily have a LongName. Register::parse parses
// OpSpecs first, then combines them with the shared long name to produce the
// final Operation.
enum OpSpec {
    Read(Span, Option<Type>),
    Write(Span, Option<Type>),
    UnsafeRead(Span),
    UnsafeWrite(Span),
}

impl Parse for OpSpec {
    fn parse(input: ParseStream) -> parse::Result<OpSpec> {
        let name: Ident = input.parse()?;
        match (&*name.to_string(), maybe_long_name(input)?) {
            ("Read", long_name) => Ok(OpSpec::Read(name.span(), long_name)),
            ("Write", long_name) => Ok(OpSpec::Write(name.span(), long_name)),
            ("UnsafeRead", None) => Ok(OpSpec::UnsafeRead(name.span())),
            ("UnsafeWrite", None) => Ok(OpSpec::UnsafeWrite(name.span())),
            _ => Err(input.error("Invalid operation")),
        }
    }
}

// Check for a long name specification at the current cursor, and if so, parse
// it. If not, returns None. Example of a long name: `(Ctrl)`
fn maybe_long_name(input: ParseStream) -> parse::Result<Option<Type>> {
    Ok(match input.peek(Paren) {
        false => None,
        true => {
            let long_name;
            parenthesized!(long_name in input);
            Some(long_name.parse()?)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn complex() {
        let parsed: Input = parse_quote! {registers;
            #[allow_bus_adapter]
            pub(crate) foo<L: LiteXRegisters, G> {
                0x00 => minimal: u32 {},
                0x04 => short_array: u8[3] { Read },
                /// Single-line doc comment.
                0x07 => long_name: u32(Ctrl) { Write },
                /// Multiline
                /// doc comment.
                0x0b => array_unsafe: u16[2] { UnsafeRead, UnsafeWrite },
                _    => aliased: u32 { Read(RegA), Write(RegB) },
            }
        };
        assert_eq!(
            parsed,
            Input {
                tock_registers: parse_quote! {registers},
                allow_bus_adapter: true,
                visibility: parse_quote! {pub(crate)},
                accessor: parse_quote! {foo},
                generics: vec![parse_quote! {L: LiteXRegisters}, parse_quote! {G}],
                registers: vec![
                    Register {
                        comments: vec![],
                        offset: Some(parse_quote! {0x00}),
                        name: parse_quote! {minimal},
                        data_type: DataType::Scalar {
                            value_type: parse_quote! {u32},
                        },
                        operations: vec![],
                    },
                    Register {
                        comments: vec![],
                        offset: Some(parse_quote! {0x04}),
                        name: parse_quote! {short_array},
                        data_type: DataType::Array {
                            value_type: parse_quote! {u8},
                            len: parse_quote! {3}
                        },
                        operations: vec![Operation::Read(Read {
                            long_name: parse_quote! {()},
                            span: Span::call_site()
                        })],
                    },
                    Register {
                        comments: vec![parse_quote! {#[doc=r" Single-line doc comment."]}],
                        offset: Some(parse_quote! {0x07}),
                        name: parse_quote! {long_name},
                        data_type: DataType::Scalar {
                            value_type: parse_quote! {u32},
                        },
                        operations: vec![Operation::Write(Write {
                            long_name: parse_quote! {Ctrl},
                            span: Span::call_site(),
                        })],
                    },
                    Register {
                        comments: vec![
                            parse_quote! {#[doc=r" Multiline"]},
                            parse_quote! {#[doc=r" doc comment."]}
                        ],
                        offset: Some(parse_quote! {0x0b}),
                        name: parse_quote! {array_unsafe},
                        data_type: DataType::Array {
                            value_type: parse_quote! {u16},
                            len: parse_quote! {2}
                        },
                        operations: vec![
                            Operation::UnsafeRead(UnsafeRead {
                                span: Span::call_site()
                            }),
                            Operation::UnsafeWrite(UnsafeWrite {
                                span: Span::call_site()
                            })
                        ],
                    },
                    Register {
                        comments: vec![],
                        offset: None,
                        name: parse_quote! {aliased},
                        data_type: DataType::Scalar {
                            value_type: parse_quote! {u32}
                        },
                        operations: vec![
                            Operation::Read(Read {
                                long_name: parse_quote! {RegA},
                                span: Span::call_site(),
                            }),
                            Operation::Write(Write {
                                long_name: parse_quote! {RegB},
                                span: Span::call_site(),
                            })
                        ],
                    }
                ],
            }
        );
    }

    #[test]
    fn const_param() {
        let parsed: syn::Result<Input> = parse2(quote! {registers; foo<const BUS_WIDTH: u8> {}});
        assert!(parsed.unwrap_err().to_string().contains("must be types"));
    }

    #[test]
    fn invalid_peripheral_attrs() {
        let parsed: syn::Result<Input> = parse2(quote! {registers; #[unknown] foo {}});
        assert!(parsed
            .unwrap_err()
            .to_string()
            .contains("Unknown attribute"));
        let parsed: syn::Result<Input> = parse2(quote! {registers; #[derive(Copy)] foo {}});
        assert!(parsed
            .unwrap_err()
            .to_string()
            .contains("Unknown attribute"));
        let parsed: syn::Result<Input> = parse2(quote! {
            registers; #[allow_bus_adapter] #[allow_bus_adapter] foo {}
        });
        assert!(parsed
            .unwrap_err()
            .to_string()
            .contains("At most one attribute"));
    }

    #[test]
    fn invalid_register_attr() {
        let parsed: syn::Result<Input> = parse2(quote! {registers; foo {
            #[test]
            0x0 => ctrl: u8 {},
        }});
        assert!(parsed
            .unwrap_err()
            .to_string()
            .contains("Attributes other than doc comments are not supported"));
    }

    #[test]
    fn invalid_op() {
        let parsed: syn::Result<Input> = parse2(quote! {registers; foo {
            0x0 => ctrl: u8 { Read, Write, IDontExist },
        }});
        assert!(parsed
            .unwrap_err()
            .to_string()
            .contains("Invalid operation"));
    }

    #[test]
    fn lifetime_param() {
        let parsed: syn::Result<Input> = parse2(quote! {registers; foo<'a> {}});
        assert!(parsed.unwrap_err().to_string().contains("must be types"));
    }

    #[test]
    fn minimal() {
        let parsed: Input = parse_quote! {registers; foo {}};
        assert_eq!(
            parsed,
            Input {
                tock_registers: parse_quote! {registers},
                allow_bus_adapter: false,
                visibility: parse_quote! {},
                accessor: parse_quote! {foo},
                generics: vec![],
                registers: vec![],
            }
        );
    }
}
