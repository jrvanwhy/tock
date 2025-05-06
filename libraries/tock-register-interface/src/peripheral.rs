// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

// TODO: Finish documentation
#[macro_export]
macro_rules! peripheral {
    [$($tokens:tt)*] => {
        $crate::reexport::peripheral!($crate; $($tokens)*);
    }
}
