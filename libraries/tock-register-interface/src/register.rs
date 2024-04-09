// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{Access, ArrayDataType, DataType, LongNames, IsSafe, IsUnsafe, UIntLike};

pub trait Register: Copy {
    type DataType: DataType;
    type LongNames: LongNames;
    type Read: Access;
    type Write: Access;

    //fn read(self) -> Self::DataType where Self::DataType: UIntLike, Self::Read: Safe;

    fn read_at(self, index: usize) -> Option<<Self::DataType as DataType>::Value>
    where
        Self::DataType: ArrayDataType,
        Self::Read: IsSafe;

    // Safety: hardware-specific requirements
    unsafe fn unsafe_read_at(self, index: usize) -> Option<<Self::DataType as DataType>::Value>
    where
        Self::DataType: ArrayDataType,
        Self::Read: IsUnsafe;

    // Safety: index < LEN
    unsafe fn read_at_unchecked(self, index: usize) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ArrayDataType,
        Self::Read: IsSafe,
    {
        assert!(
            index < Self::DataType::LEN,
            "index out of bounds: the len is {} but the index is {}",
            Self::DataType::LEN,
            index
        );
        self.read_at(index)
            .expect("read_at called with in-bounds index but returned None")
    }

    // Safety: index < LEN and hardware-specific requirements
    unsafe fn unsafe_read_at_unchecked(self, index: usize) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ArrayDataType,
        Self::Read: IsUnsafe,
    {
        assert!(
            index < Self::DataType::LEN,
            "index out of bounds: the len is {} but the index is {}",
            Self::DataType::LEN,
            index
        );
        unsafe { self.unsafe_read_at(index) }
            .expect("read_at called with in-bounds index but returned None")
    }
}
