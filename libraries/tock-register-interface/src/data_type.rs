// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::UIntLike;

pub trait ArrayDataType: DataType {
    const LEN: usize;
}
impl<U: UIntLike, const LEN: usize> ArrayDataType for [U; LEN] {
    const LEN: usize = LEN;
}

pub trait DataType: private::Sealed {
    type Value: UIntLike;
}

impl<U: UIntLike> DataType for U {
    type Value = U;
}
impl<U: UIntLike> private::Sealed for U {}

impl<U: UIntLike, const LEN: usize> DataType for [U; LEN] {
    type Value = U;
}
impl<U: UIntLike, const LEN: usize> private::Sealed for [U; LEN] {}

mod private {
    pub trait Sealed {}
}
