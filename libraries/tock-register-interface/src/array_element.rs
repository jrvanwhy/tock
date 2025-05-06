// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{
    ArrayRead, ArrayRegister, ArrayUnsafeRead, ArrayUnsafeWrite, ArrayWrite, Read, ReadLongName,
    Register, UnsafeRead, UnsafeWrite, Write, WriteLongName,
};

/// A reference to an element of an array type register. This reference
/// implements `Read`, `Write`, `UnsafeRead`, and/or `UnsafeWrite`, depending on
/// the array definition in the `peripheral!` invocation.
#[derive(Clone)]
pub struct ArrayElement<R: ArrayRegister> {
    // Safety invariant: index < R::LEN.
    index: usize,
    register: R,
}

impl<R: ArrayRegister> ArrayElement<R> {
    /// Code outside this crate should use ArrayRegister::get() and/or ArrayIter
    /// to construct ArrayElements.
    pub(crate) fn new(register: R, index: usize) -> Option<ArrayElement<R>> {
        if index >= R::LEN {
            return None;
        }
        Some(ArrayElement {
            // Safety: We just confirmed that index < R::LEN with the above if
            // statement.
            index,
            register,
        })
    }
}

impl<R: ArrayRegister> Register for ArrayElement<R> {
    type Value = R::Value;
}

impl<R: ArrayRegister + ReadLongName> ReadLongName for ArrayElement<R> {
    type LongName = R::LongName;
}

impl<R: ArrayRegister + ArrayRead> Read for ArrayElement<R> {
    fn read(&self) -> Self::Value {
        // Safety: self.index < R::LEN is the safety invariant on self.index.
        unsafe { self.register.read_unchecked(self.index) }
    }
}

impl<R: ArrayRegister + WriteLongName> WriteLongName for ArrayElement<R> {
    type LongName = R::LongName;
}

impl<R: ArrayRegister + ArrayWrite> Write for ArrayElement<R> {
    fn write(&self, value: Self::Value) {
        // Safety: self.index < R::LEN is the safety invariant on self.index.
        unsafe {
            self.register.write_unchecked(self.index, value);
        }
    }
}

impl<R: ArrayRegister + ArrayUnsafeRead> UnsafeRead for ArrayElement<R> {
    unsafe fn read(&self) -> Self::Value {
        // Safety:
        // 1. `self.index < R::LEN` is the safety invariant on self.index.
        // 2. The caller has complied with the hardware-specific safety
        //    invariants (that is a precondition of UnsafeRead::read).
        unsafe { self.register.read_unchecked(self.index) }
    }
}

impl<R: ArrayRegister + ArrayUnsafeWrite> UnsafeWrite for ArrayElement<R> {
    unsafe fn write(&self, value: Self::Value) {
        // Safety:
        // 1. `index < R::LEN` is the safety invariant on self.index.
        // 2. The caller has complied with the hardware-specific safety
        //    invariants (that is a precondition of UnsafeWrite::write).
        unsafe {
            self.register.write_unchecked(self.index, value);
        }
    }
}
