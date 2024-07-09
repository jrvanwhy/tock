// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{ArrayElement, ArrayRegister};

/// An iterator over elements of an array register.
pub struct ArrayIter<R: ArrayRegister> {
    index: usize,
    register: R,
}

impl<R: ArrayRegister> ArrayIter<R> {
    /// Constructs a new ArrayIter pointing to the given register. Most callers
    /// should use .into_iter() on the register itself to create an
    /// ArrayRegister.
    pub fn new(register: R) -> ArrayIter<R> {
        ArrayIter { index: 0, register }
    }
}

impl<R: ArrayRegister> Iterator for ArrayIter<R> {
    type Item = ArrayElement<R>;

    fn next(&mut self) -> Option<ArrayElement<R>> {
        let out = self.register.get(self.index)?;
        self.index += 1;
        Some(out)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<ArrayElement<R>> {
        if self.index >= R::LEN {
            return None;
        }
        self.register.get(R::LEN - 1)
    }

    fn nth(&mut self, n: usize) -> Option<ArrayElement<R>> {
        let Some(out) = self.register.get(self.index + n) else {
            // If you call nth() with an n >= len(), nth() should exhaust the
            // entire iterator and return None.
            self.index = R::LEN;
            return None;
        };
        self.index += n + 1;
        Some(out)
    }
}

impl<R: ArrayRegister> ExactSizeIterator for ArrayIter<R> {
    // This is a more efficient implementation of len() than ExactSizeIterator
    // provides.
    fn len(&self) -> usize {
        R::LEN - self.index
    }
}

#[cfg(test)]
mod tests {
    use crate::{peripheral, Read};

    peripheral! {
        Foo {
            _ => a: u32[2] { Read },
            _ => empty: u8[0] {},
        }
    }

    #[derive(Clone)]
    struct FakeFoo;
    impl Foo for FakeFoo {
        fn a_read(&self, index: usize) -> Option<u32> {
            [3, 4].get(index).copied()
        }
    }

    #[test]
    fn count() {
        let peripheral = Peripheral::new(FakeFoo);
        assert_eq!(peripheral.a().into_iter().count(), 2);
        let mut iter = peripheral.a().into_iter();
        iter.next();
        assert_eq!(iter.count(), 1);
        let mut iter = peripheral.a().into_iter();
        iter.next();
        iter.next();
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn last() {
        let peripheral = Peripheral::new(FakeFoo);
        assert_eq!(peripheral.a().into_iter().last().unwrap().read(), 4);
        assert!(peripheral.empty().into_iter().last().is_none());
    }

    #[test]
    fn len() {
        let peripheral = Peripheral::new(FakeFoo);
        let mut iter = peripheral.a().into_iter();
        assert_eq!(iter.len(), 2);
        iter.next();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
        // The next check verifies that the substraction does not underflow.
        iter.next();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn next() {
        let peripheral = Peripheral::new(FakeFoo);
        let mut iter = peripheral.a().into_iter();
        let first = iter.next().unwrap();
        assert_eq!(first.read(), 3);
        let second = iter.next().unwrap();
        assert_eq!(second.read(), 4);
        assert!(iter.next().is_none());
    }

    #[test]
    fn nth() {
        let peripheral = Peripheral::new(FakeFoo);
        assert_eq!(peripheral.a().into_iter().nth(0).unwrap().read(), 3);
        assert_eq!(peripheral.a().into_iter().nth(1).unwrap().read(), 4);
        assert!(peripheral.a().into_iter().nth(2).is_none());
        let mut iter = peripheral.a().into_iter();
        assert_eq!(iter.nth(1).unwrap().read(), 4);
        assert_eq!(iter.len(), 0);
        let mut iter = peripheral.a().into_iter();
        assert!(iter.nth(2).is_none());
        assert_eq!(iter.len(), 0);
        let mut iter = peripheral.a().into_iter();
        assert!(iter.nth(3).is_none());
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn size_hint() {
        let peripheral = Peripheral::new(FakeFoo);
        let mut iter = peripheral.a().into_iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
