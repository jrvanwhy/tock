// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

//! Register traits implemented for array-type registers. Array registers are
//! treated as arrays of normal registers. To interact with one, retrieve a
//! reference to the element you want to use using [`ArrayRegister::get()`],
//! then use the returned [`ArrayElement`] to interact with that element.

use crate::register_traits::{ReadLongName, Register, WriteLongName};
use crate::ArrayElement;

pub trait ArrayRegister: Register {
    const LEN: usize;

    /// Returns a reference a particular element of this array.
    fn get(&self, index: usize) -> Option<ArrayElement<Self>> {
        ArrayElement::new(self.clone(), index)
    }
}

/// Implemented by array registers that are safely readable. Normally used via
/// [`ArrayElement`].
pub trait ArrayRead: ArrayRegister + ReadLongName {
    // read_unchecked allows ArrayElement to perform reads without redundant
    // bounds checks.
    /// Read the value at the specified index.
    /// # Safety
    /// Precondition: `index < Self::LEN`.
    unsafe fn read_unchecked(&self, index: usize) -> Self::Value;
}

/// Implemented by array registers that are safely writable. Normally used via
/// [`ArrayElement`].
pub trait ArrayWrite: ArrayRegister + WriteLongName {
    // write_unchecked allows ArrayElement to perform writes without redundant
    // bounds checks.
    /// Write a value at the specified index.
    /// # Safety
    /// Precondition: `index < Self::LEN`.
    unsafe fn write_unchecked(&self, index: usize, value: Self::Value);
}

/// Implemented by array registers that are readable but not safely readable.
/// Normally used via [`ArrayElement`].
pub trait ArrayUnsafeRead: ArrayRegister {
    // read_unchecked allows ArrayElement to perform reads without redundant
    // bounds checks.
    /// Read the value at the specified index.
    /// # Safety
    /// Precondition: `index < Self::LEN`.
    /// Reading this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn read_unchecked(&self, index: usize) -> Self::Value;
}

/// Implemented by array registers that are writable but not safely writable.
/// Normally used via [`ArrayElement`].
pub trait ArrayUnsafeWrite: ArrayRegister {
    // write_unchecked allows ArrayElement to perform writes without redundant
    // bounds checks.
    /// Write a value at the specified index.
    /// # Safety
    /// Precondition: `index < Self::LEN`.
    /// Reading this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn write_unchecked(&self, index: usize, value: Self::Value);
}
