// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{BusAdapter, OutOfBounds, UIntLike};

/// A pointer into MMIO memory.
/// # Safety
/// Each MmioPointer instance must have a single pointer value, and `pointer`
/// must return that value.
pub unsafe trait MmioPointer: Copy {
    fn pointer(self) -> *mut ();

    // TODO: Add test
    /// Reads an element out of an array-type register. `OFFSET` is location of
    /// the start of the register relative to the value of this `MmioPointer`,
    /// `LEN` is the number of elements in the array, and `Value` is the type of
    /// the elements in the array.
    /// Returns `None` if `index` is out of bounds (`>= LEN`).
    /// # Safety
    /// `OFFSET`, `LEN`, and `Value` must accurately describe an MMIO array
    /// register contained in the MMIO peripheral that this `MmioPointer` points
    /// to. `bus` must be a correct `BusAdapter` for the MMIO bus this register
    /// is on. If this register has hardware-specific safety requirements, the
    /// caller must comply with those requirements.
    unsafe fn array_read<
        Bus: BusAdapter<Value>,
        Value: UIntLike,
        const OFFSET: usize,
        const LEN: usize,
    >(
        self,
        bus: Bus,
        index: usize,
    ) -> Option<Value> {
        if !(index < LEN) {
            return None;
        }
        let pointer = self.pointer();
        let element_offset = OFFSET + Bus::SIZE * index;
        // Safety:
        // Because this array register is part of the MMIO peripheral that
        // `pointer` points to, and we confirmed `index` is in-bounds,
        // element_offset falls within the same allocation as `pointer`. The
        // caller guaranteed tha `bus` is the correct `BusAdapter` for this
        // register, and that there is an array-typed register at the given
        // location. The caller has complied with any hardware-specific safety
        // requirements.
        Some(unsafe { bus.read(pointer.byte_add(element_offset)) })
    }

    // TODO: Add test
    /// Reads an element out of an array-type register, without bounds checking.
    /// `OFFSET` is location of the start of the register relative to the value
    /// of this `MmioPointer`, and `Value` is the type of the elements in the
    /// array.
    /// # Safety
    /// `OFFSET`, and `Value` must accurately describe an MMIO array register
    /// contained in the MMIO peripheral that this `MmioPointer` points to.
    /// `index` must be an in-bounds index for that array. `bus` must be a
    /// correct `BusAdapter` for the MMIO bus this register is on. If this
    /// register has hardware-specific safety requirements, the caller must
    /// comply with those requirements.
    unsafe fn array_read_unchecked<Bus: BusAdapter<Value>, Value: UIntLike, const OFFSET: usize>(
        self,
        bus: Bus,
        index: usize,
    ) -> Value {
        let pointer = self.pointer();
        let element_offset = OFFSET + Bus::SIZE * index;
        // Safety:
        // Because this array register is part of the MMIO peripheral that
        // `pointer` points to, and the caller guaranteed `index` is in-bounds,
        // element_offset falls within the same allocation as `pointer`. The
        // caller guaranteed tha `bus` is the correct `BusAdapter` for this
        // register, and that there is an array-typed register at the given
        // location. The caller has complied with any hardware-specific safety
        // requirements.
        Some(unsafe { bus.read(pointer.byte_add(element_offset)) })
    }

    // TODO: Doc, including Safety doc.
    unsafe fn array_write<
        Bus: BusAdapter<Value>,
        Value: UIntLike,
        const OFFSET: usize,
        const LEN: usize,
    >(
        self,
        bus: Bus,
        index: usize,
        value: Value,
    ) -> Result<(), OutOfBounds> {
        if !(index < LEN) {
            return None;
        }
        let pointer = self.pointer();
        let element_offset = OFFSET + Bus::SIZE * index;
        // TODO: Safety comment
        Some(unsafe { bus.write(pointer.byte_add(element_offset), value) })
    }

    // TODO: Doc, including safety doc.
    unsafe fn array_write_unchecked<
        Bus: BusAdapter<Value>,
        Value: UIntLike,
        const OFFSET: usize,
    >(
        self,
        bus: Bus,
        index: usize,
        value: Value,
    ) {
        let pointer = self.pointer();
        let element_offset = OFFSET + Bus::SIZE * index;
        // TODO: Safety comment
        Some(unsafe { bus.write(pointer.byte_add(element_offset), value) })
    }

    // TODO: Doc, including safety doc
    unsafe fn read<Bus: BusAdapter<Value>, Value: UIntLike, const OFFSET: usize>(
        self,
        bus: Bus,
    ) -> Value {
        let pointer = self.pointer();
        // TODO: Safety doc
        Some(unsafe { bus.read(pointer.byte_add(OFFSET)) })
    }

    // TODO: Doc, including safety doc
    unsafe fn read<Bus: BusAdapter<Value>, Value: UIntLike, const OFFSET: usize>(
        self,
        bus: Bus,
        value: Value,
    ) {
        let pointer = self.pointer();
        // TODO: Safety doc
        Some(unsafe { bus.write(pointer.byte_add(OFFSET), value) })
    }
}

/// A MmioPointer whose value is stored in memory.
#[derive(Clone, Copy)]
pub struct DynPointer {
    pointer: *mut (),
}

impl DynPointer {
    pub fn new(pointer: *mut ()) -> DynPointer {
        DynPointer { pointer }
    }
}

unsafe impl MmioPointer for DynPointer {
    fn pointer(self) -> *mut () {
        self.pointer
    }
}

/// A zero-sized MmioPointer whose value is const.
#[derive(Clone, Copy)]
pub struct ConstPointer<const ADDRESS: usize>;

impl<const ADDRESS: usize> ConstPointer<ADDRESS> {
    // There is not currently a strict-provenance-compatible way to create a
    // pointer to an MMIO peripheral [1]. While we wait for one to be
    // stabilized, we'll just perform the cast at `const` time and hope that is
    // sufficient for any Rust compilers targetting a CHERI platform to generate
    // a valid pointer.
    // [1] https://github.com/rust-lang/rust/issues/98593
    const POINTER: *mut () = ADDRESS as *mut ();
}

unsafe impl<const ADDRESS: usize> MmioPointer for ConstPointer<ADDRESS> {
    fn pointer(self) -> *mut () {
        Self::POINTER
    }
}
