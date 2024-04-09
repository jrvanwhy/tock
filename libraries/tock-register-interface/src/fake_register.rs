// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{Access, ArrayDataType, DataType, LongNames, IsSafe, IsUnsafe, OutOfBounds, Register};
use core::marker::PhantomData;

pub struct FakeRegister<Data: Copy, DT: DataType, LN: LongNames, Read: Access, Write: Access> {
    data: Data,
    _phantom: PhantomData<(LN, Read, Write)>,
    // index ignored if not an array. These functions are bounds checked. If
    // scalar typed, always return Some/Ok.
    read: fn(Data, usize) -> Option<DT::Value>,
    write: fn(Data, usize, DT::Value) -> Result<(), OutOfBounds>,
}

impl<Data: Copy, DT: DataType, LN: LongNames, Read: Access, Write: Access> Clone
    for FakeRegister<Data, DT, LN, Read, Write>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<Data: Copy, DT: DataType, LN: LongNames, Read: Access, Write: Access> Copy
    for FakeRegister<Data, DT, LN, Read, Write>
{
}

impl<Data: Copy, DT: DataType, LN: LongNames, Read: Access, Write: Access> Register
    for FakeRegister<Data, DT, LN, Read, Write>
{
    type DataType = DT;
    type LongNames = LN;
    type Read = Read;
    type Write = Write;

    fn read_at(self, index: usize) -> Option<DT::Value>
    where
        DT: ArrayDataType,
        Read: IsSafe,
    {
        if index >= DT::LEN {
            return None;
        }
        unsafe { (self.read)(self.data, index) }
            .or_else(|| panic!("read() returned None at in-bounds index {}", index))
    }

    unsafe fn unsafe_read_at(self, index: usize) -> Option<DT::Value>
    where
        DT: ArrayDataType,
        Read: IsUnsafe,
    {
        if index >= DT::LEN {
            return None;
        }
        unsafe { (self.read)(self.data, index) }
            .or_else(|| panic!("read() returned None at in-bounds index {}", index))
    }
}
