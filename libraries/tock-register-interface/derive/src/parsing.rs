// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{Access, Field, Input, Register};
use quote::format_ident;
use syn::parse::{self, Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::{braced, parenthesized, parse2, parse_quote, Attribute, Error, Ident, Meta, Token, Type};

// TODO: Refactor into separate parsing and validation passes. Validation should
// be able to return multiple errors at once.

// tock_registers; #[]* pub Foo { ... }
impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Input> {
        let tock_registers = input.parse()?;
        input.parse::<Token![;]>()?;
        let mut allow_bus_adapter = false;
        let mut cfgs = vec![];
        let mut comments = vec![];
        let mut real_name = None;
        for attr in input.call(Attribute::parse_outer)? {
            match attr.meta {
                Meta::List(ref list) if list.path.is_ident("cfg") => cfgs.push(attr),
                Meta::List(list) if list.path.is_ident("real") => {
                    if real_name.is_some() {
                        return Err(Error::new_spanned(list, "duplicate #[real()] attribute"));
                    }
                    real_name = Some(parse2(list.tokens)?);
                }
                Meta::NameValue(ref name_value) if name_value.path.is_ident("doc") => {
                    comments.push(attr)
                }
                Meta::Path(ref path) if path.is_ident("allow_bus_adapter") => {
                    if allow_bus_adapter {
                        return Err(Error::new_spanned(
                            path,
                            "duplicate allow_bus_adapter attributes",
                        ));
                    }
                    allow_bus_adapter = true;
                }
                _ => return Err(Error::new_spanned(attr.meta, "unknown attribute")),
            }
        }
        let visibility = input.parse()?;
        let name = input.parse()?;
        let real_name = real_name.unwrap_or_else(|| format_ident!("Mmio{}", name));
        let fields;
        braced!(fields in input);
        let fields = fields
            .parse_terminated(Field::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Input {
            allow_bus_adapter,
            cfgs,
            comments,
            fields,
            name,
            real_name,
            tock_registers,
            visibility,
        })
    }
}

// #[...]* followed by
// _ => _: [u8; 4],
// 1 => _: u32,
// _ => ctrl: u32 { Read, Write },
// 2 => ctrl: [u16; 2] {},
impl Parse for Field {
    fn parse(input: ParseStream) -> parse::Result<Field> {
        let mut cfgs = vec![];
        let mut comments = vec![];
        for attr in input.call(Attribute::parse_outer)? {
            match attr.meta {
                Meta::List(ref list) if list.path.is_ident("cfg") => cfgs.push(attr),
                Meta::NameValue(ref name_value) if name_value.path.is_ident("doc") => {
                    comments.push(attr)
                }
                _ => return Err(Error::new_spanned(attr, "unknown attribute")),
            }
        }
        let offset = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            Some(input.parse()?)
        };
        input.parse::<Token![=>]>()?;
        let name = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            Some(input.parse()?)
        };
        input.parse::<Token![:]>()?;
        let data_type = input.parse()?;
        let shared_long_name = maybe_long_name(input)?;
        let Some(name) = name else {
            // Just a padding field
            if let Some(long_name) = shared_long_name {
                // TODO: Test this error message.
                return Err(Error::new_spanned(
                    long_name,
                    "Padding cannot have a LongName",
                ));
            }
            return Ok(Field {
                cfgs,
                comments,
                data_type,
                offset,
                register: None,
            });
        };
        let op_specs;
        braced!(op_specs in input);
        // (Access, long_name: Option<Type>>
        let mut read = (Access::NoAccess, None);
        let mut write = (Access::NoAccess, None);
        for op_spec in op_specs.parse_terminated(OpSpec::parse, Token![,])? {
            let (var, access) = match op_spec.name {
                name if name == "Read" => (&mut read, Access::Safe(name)),
                name if name == "UnsafeRead" => (&mut read, Access::Unsafe(name)),
                name if name == "Write" => (&mut write, Access::Safe(name)),
                name if name == "UnsafeWrite" => (&mut write, Access::Unsafe(name)),
                name => return Err(Error::new_spanned(name, "Unknown operation")),
            };
            match var {
                (Access::Safe(name), _) | (Access::Unsafe(name), _) => {
                    return Err(Error::new_spanned(
                        name,
                        "multiple operations of the same type specified",
                    ))
                }
                _ => {}
            }
            *var = (access, op_spec.long_name);
        }
        let (long_names, read, write) = match (shared_long_name, read, write) {
            _ => todo!("Type combo not implemented yet"),
        };
        Ok(Field {
            cfgs,
            comments,
            data_type,
            offset,
            register: Some(Register {
                name,
                long_names,
                read,
                write,
            }),
        })
    }
}

struct OpSpec {
    name: Ident,
    long_name: Option<Type>,
}

impl Parse for OpSpec {
    fn parse(input: ParseStream) -> parse::Result<OpSpec> {
        Ok(OpSpec {
            name: input.parse()?,
            long_name: maybe_long_name(input)?,
        })
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
