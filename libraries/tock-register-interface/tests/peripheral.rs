// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

// TODO: Add a #![no_implicit_prelude] test.

tock_registers::peripheral! {
    //#[allow_bus_adapter]
    Foo {
        0x0 => safe_scalar_readonly: u8 { Read },
        0x1 => a: u16[3] { Write },
        0x7 => b: u32 {},
    }
}
