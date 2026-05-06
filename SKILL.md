---
name: peat-lite
description: Per-repo skill for the Peat lightweight CRDT primitives — no_std-capable core for resource-constrained nodes plus Android UniFFI bindings.
when_to_use: Editing files under peat-lite/, reviewing peat-lite PRs, debugging CRDT or wire-format issues, working on the no_std embedded build, working on the Android FFI surface, or working on the ESP32 firmware.
verifies_with: cargo fmt --check, cargo clippy -- -D warnings, cargo test (std), cargo build --no-default-features (no_std), cargo build -p peat-lite-android (UniFFI cdylib), plus firmware/ and fuzz/ builds where touched.
---

# `peat-lite` SKILL

`peat-lite` is the lightweight CRDT building block of the Peat ecosystem. The core crate is `no_std`-capable and intentionally minimal-dependency (only `heapless` for fixed-capacity collections) so it can run on resource-constrained nodes (M5Stack Core2, ESP32-class hardware) alongside the BLE transport from `peat-btle`. The repo is a Cargo workspace: the core `peat-lite` crate plus an `android-ffi` member that wraps the same primitives in UniFFI for Android consumers. The `firmware/` (ESP32 OTA) and `fuzz/` (cargo-fuzz harness) directories are workspace-excluded — they have their own `Cargo.toml` and toolchains.

## When this skill applies

- Editing any file under `src/` (CRDT logic, wire format)
- Editing `android-ffi/` (UniFFI surface for Android consumers)
- Editing `firmware/` (ESP32 OTA build) — note this is a workspace-excluded crate with its own `Cargo.toml` and `rust-toolchain.toml`
- Editing `fuzz/` (cargo-fuzz harness) — also workspace-excluded
- Touching the feature-flag matrix in `Cargo.toml` (`std` vs. no-default-features split)
- Working on integration with `peat-btle` (BLE transport pairs with `peat-lite` for embedded consumers)

## Scope

**In scope:**
- CRDT primitives and wire-format encode/decode in the core crate
- The `no_std` boundary — what compiles without std, what requires it
- Android FFI surface in `android-ffi/` (UniFFI-generated Kotlin bindings)
- ESP32 firmware in `firmware/` (separate toolchain, separate dependency tree)
- Fuzzing harness in `fuzz/` (cargo-fuzz)

**Out of scope (route elsewhere):**
- BLE transport implementation → `peat-btle/SKILL.md`
- Mesh routing / topology / sync → `peat-mesh/SKILL.md` (the heavier sibling)
- Top-level shared types/traits — consider whether the change belongs in `peat/peat-protocol` or `peat/peat-schema` (workspace subcrates of the `peat` repo)

## Workflow

1. **Orient.** Read `peat/SKILL.md` (ecosystem) if accessible. Read this file. Read `CONTRIBUTING.md`. `git status`, `git log -10`.
2. **Locate the spec.** Confirm the task has a GitHub issue with Context / Scope / Acceptance / Constraints / Dependencies. If not, stop and ask the user.
3. **Plan.** Produce a 1–5 step plan. Cross-check against ecosystem hard invariants (transport agnosticism, dependency direction, no_std discipline) and the scope guards below.
4. **Implement.** Branch from `main` per the trunk-based convention. Vertical slices, one concern per commit. Keep core code `no_std`-clean unless the change is gated behind the `std` feature.
5. **Verify.** Run every command in the verification checklist below. Capture output.
6. **Hand off.** Open PR against `main` referencing the issue. Single concern per PR — squash-merge applies. `CODEOWNERS` will route review.

## Verification (exit criteria)

A session in this repo is not done until each of these produces evidence:

- [ ] `cargo fmt --check` exits 0
- [ ] `cargo clippy -- -D warnings` exits 0
- [ ] `cargo test` exits 0 (default features = `std`)
- [ ] `cargo build --no-default-features` exits 0 — verifies the core stays `no_std`-compatible
- [ ] If `android-ffi/` was touched: `cargo build -p peat-lite-android` exits 0; if the UniFFI surface changed, regenerated bindings are committed in the same PR
- [ ] If `firmware/` was touched: build the firmware crate from its own directory with the ESP32 toolchain (`cd firmware && cargo build` against its pinned `rust-toolchain.toml`)
- [ ] If `fuzz/` was touched: at least one fuzzer target builds (`cd fuzz && cargo +nightly fuzz build <target>`)

"Seems right" or "the diff looks correct" is never sufficient.

## Anti-rationalization

| Excuse | Rebuttal |
|---|---|
| "This change is too small to need a test." | If it's worth changing, it's worth one assertion. Add the test. |
| "I'll fix the clippy warning later." | The CI gate is `-D warnings`. There is no later. Fix it before commit. |
| "I'll just import `std::collections::HashMap`; it's only one place." | The core crate is `no_std`-capable on purpose — that's why it can run on M5Stack/ESP32. Use `heapless` collections, or gate std-using code behind the `std` feature. |
| "Pulling in one more dep won't bloat embedded builds." | peat-lite is intentionally minimal-dep (just `heapless`). Every new dep needs explicit justification — embedded targets care about both binary size and compile time. |
| "I'll regenerate the UniFFI bindings in a follow-up." | Stale bindings break Android consumers. Regenerate and commit in the same PR. |
| "The firmware build is its own thing — I don't need to verify it." | If you touched `peat-lite` core API, the firmware that consumes it can break in ways `cargo test` won't catch. Build the firmware crate when the public surface changes. |
| "`fuzz/` is excluded, so I can ignore it." | The fuzzers exist because wire-format decode bugs are reachable from untrusted input. If you change decode, build the fuzzer; if it fails to build, you've changed the wire contract. |

## Scope guards

- Touch only files the issue/user asked you to touch.
- Do not edit other peat-* repos. Cross-repo work goes in a separate PR in that repo, linked through a tracking issue.
- Keep the core crate `no_std`-clean unless the change is explicitly gated behind the `std` feature. Public types should be available to no_std consumers wherever possible.
- Do not add dependencies to the core crate without explicit user approval. The `heapless`-only dep tree is a feature, not an oversight.
- Do not bleed Android/UniFFI specifics into the core crate. Keep them inside `android-ffi/`.
- Do not configure git to bypass GPG signing or use `--no-verify` to skip pre-commit hooks.
- Review routing follows `CODEOWNERS` (`@defenseunicorns/peat`).

## Gotchas

Add an entry each time a session produces output that needed correction. One line per gotcha plus a `Why:` line.

- *(none recorded yet)*

## References (read on demand, not by default)

- Ecosystem invariants: `peat/SKILL.md` (sibling repo)
- Build matrix: `CONTRIBUTING.md`
- `CODEOWNERS` — team-level routing (`* @defenseunicorns/peat`) plus privileged-file restrictions on `CODEOWNERS` and `LICENSE`
- Repo: https://github.com/defenseunicorns/peat-lite

---
*Last updated: 2026-05-05*
*Maintained by: Kit Plummer, VP Data and Autonomy, Defense Unicorns*
