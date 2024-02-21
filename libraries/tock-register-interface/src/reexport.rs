// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

//! Re-exports of items from external crates, for use by tock-registers' macros.
//! Not for use by outside crates (the contents of this module are not stable).

#![doc(hidden)]

pub use core::assert;
pub use core::mem::size_of; // TODO: Needed after LiteX support?
pub use core::unimplemented;
pub use tock_registers_derive::peripheral;
pub use IntoIterator;
pub use Option;
pub use Result;
