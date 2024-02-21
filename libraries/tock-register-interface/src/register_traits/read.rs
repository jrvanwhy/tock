// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::register_traits::ReadLongName;

// TODO: Document
pub trait Read: ReadLongName {
    // TODO: Document
    fn read(&self) -> Self::Value;

    // TODO: Other operations.
}
