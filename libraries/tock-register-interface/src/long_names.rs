// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2024.

/// Descriptive name for each register.
pub trait RegisterLongName {}

// Useful implementation for when no RegisterLongName is required
// (e.g. no fields need to be accessed, just the raw register values)
impl RegisterLongName for () {}

pub trait LongNames {
    type Read: RegisterLongName;
    type Write: RegisterLongName;
}

impl<L: RegisterLongName> LongNames for L {
    type Read = L;
    type Write = L;
}
