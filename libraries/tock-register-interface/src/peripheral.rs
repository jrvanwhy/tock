// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

// TODO: Finish documentation
#[macro_export]
// TODO: Allow declaring multiple peripherals.
macro_rules! peripheral {
    [$($(#[$attr:meta])* $visibility:vis $name:ident {$($fields:tt)*})*] => {$(
        $crate::reexport::peripheral!($crate; $(#[$attr])* $visibility $name { $($fields)* });
    )*}
}
