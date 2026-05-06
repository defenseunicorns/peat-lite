# CLAUDE.md — `peat-lite`

Before doing any work in this repo, read **both** of:

1. `SKILL.md` (this repo) — the per-repo workflow, verification checklist, and scope guards.
2. `peat/SKILL.md` (in the sibling `peat` repo, if checked out alongside) — the ecosystem skill: hard invariants, FFI conventions, the skill router across all peat-* repos.

If `peat/SKILL.md` isn't accessible, say so before proceeding — most architectural invariants live there, not here.

## Quick orientation

- **Repo role:** Lightweight CRDT primitives for resource-constrained Peat nodes. `no_std`-capable core with an optional `std` feature. Targets: host (Rust library), Android (via the `android-ffi` workspace member, UniFFI-bound), ESP32 firmware (`firmware/`, workspace-excluded with its own toolchain), fuzzing harness (`fuzz/`, workspace-excluded).
- **Primary language:** Rust (edition 2021, MSRV 1.70). Core `crate-type = ["rlib"]`; `android-ffi` adds `cdylib`.
- **Cheap sanity check:** `cargo build` (std default). For embedded coverage before pushing: `cargo build --no-default-features`.

## Hard rule

A task in this repo is not done until the verification checklist in `SKILL.md` produces evidence. "Seems right" or "the diff looks correct" is never sufficient.

GPG-signed commits are required by repo policy. Cross-repo changes require one PR per repo, linked through a tracking issue — not a single PR that reaches across repos.
