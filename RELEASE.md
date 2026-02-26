# Release Process

## Overview

Releases are automated via GitHub Actions. Pushing a version tag triggers CI,
publishes to crates.io, and creates a GitHub Release with auto-generated notes.

## Prerequisites

- `CARGO_REGISTRY_TOKEN` secret configured in GitHub repo settings
- Write access to push tags to `main`
- Version in `Cargo.toml` matches the tag you intend to push

## Steps

### 1. Prepare the release

Update the version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

Commit, push a PR, and merge to `main`:

```bash
git checkout -b release/vX.Y.Z
# edit Cargo.toml version
git add Cargo.toml
git commit -S -m "chore: bump version to X.Y.Z"
git push -u origin release/vX.Y.Z
gh pr create --title "chore: release vX.Y.Z"
# wait for CI, then merge
```

### 2. Tag and push

After the PR is merged, tag `main` and push:

```bash
git checkout main
git pull origin main
git tag -s vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

### 3. Automated pipeline

The `v*` tag push triggers `.github/workflows/release.yml` which:

1. **CI** — runs fmt, clippy, test, and no_std build (reuses `ci.yml`)
2. **Publish** — validates tag matches `Cargo.toml` version, then runs `cargo publish`
3. **GitHub Release** — creates a release with auto-generated notes from commit history

### 4. Verify

- Check the [Actions tab](https://github.com/defenseunicorns/peat-lite/actions) for workflow status
- Confirm the crate appears on [crates.io/crates/peat-lite](https://crates.io/crates/peat-lite)
- Confirm the [GitHub Release](https://github.com/defenseunicorns/peat-lite/releases) was created

## Hotfix releases

For patch releases, branch from the release tag:

```bash
git checkout -b fix/description vX.Y.Z
# make fixes, bump patch version in Cargo.toml
# PR → merge → tag → push tag
```

## Version policy

- **Major**: breaking wire protocol or public API changes
- **Minor**: new features, backward-compatible additions
- **Patch**: bug fixes, documentation, internal changes
