// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

//! UniFFI bindgen CLI entry point.
//!
//! This binary is used to generate Kotlin/Swift bindings from the
//! compiled native library.

fn main() {
    uniffi::uniffi_bindgen_main()
}
