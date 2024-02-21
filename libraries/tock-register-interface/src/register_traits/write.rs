// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::register_traits::WriteLongName;

// TODO: Document
pub trait Write: WriteLongName {
    // TODO: Document
    fn write(&self, value: Self::Value);

    // TODO: Other operations.
}
