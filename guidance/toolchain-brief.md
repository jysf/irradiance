# Toolchain Brief

> **Per-repo toolchain facts a cold build sub-agent needs.** A fresh
> build/verify sub-agent re-imports its model's generic tool-priors and burns
> loops rediscovering this repo's specifics. **Inject this into every build
> prompt** (AGENTS.md §15 "During build"; DEC-004 rule 5). If a fact here goes
> stale a sub-agent will trust it and waste the loop anyway — so prune
> aggressively.

**Every fact below was measured on 2026-08-15/16 by running the command, not
assumed.** Host: the maintainer's Mac, `aarch64-apple-darwin`. These are **host**
facts, not repo facts — re-verify on any other machine, and expect CI to differ.

---

## ⚠ Read this one first: the `+toolchain` trap

```
$ cargo +nightly --version
error: no such command: `+nightly`
```

`cargo` on `PATH` is **Homebrew's real cargo**, not a rustup shim, because
`/opt/homebrew/bin` precedes `~/.cargo/bin`. Homebrew's cargo does not understand
`+toolchain` syntax. rustup **is** installed, and its default toolchain is
nightly.

| Invocation | Resolves to |
|---|---|
| `cargo` | Homebrew **1.97.1** (`/opt/homebrew/bin/cargo`) |
| `~/.cargo/bin/cargo` | rustup shim → **nightly 1.99.0** (the default toolchain) |
| `~/.cargo/bin/cargo +stable` | **1.97.0** — a *different build* from Homebrew's 1.97.1 |

**Anything needing nightly must go through the shim explicitly:**

```bash
~/.cargo/bin/cargo +nightly fuzz run <target>
```

This costs a loop every single time it is rediscovered. `cargo fuzz` needs
nightly, and fuzz targets ship with the first parser spec (AGENTS.md §12), so
this will come up early and often.

**It has now cost three, and they are numbered below** — the plain `+nightly`
form above, `cargo fuzz`'s inner shell-out (second), and the MSRV gate's
`+1.90.0` (third). Every one was measured, not predicted. Read all three before
typing a `+` after `cargo`.

## ⚠ The SECOND `+toolchain` trap: `cargo fuzz`

The PATH fix above is not enough for fuzzing. `cargo fuzz` **shells out to a bare
`"cargo" "build"`**, and that inner call resolves to Homebrew's stable cargo,
which rejects the sanitizer flags:

```
error: 1 nightly option were parsed
Error: failed to build fuzz script
```

Even `~/.cargo/bin/cargo +nightly fuzz run` fails — the *outer* command is fine,
the *inner* one is not. Put the rustup shim first on PATH so both resolve:

```bash
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run <target>
```

Measured 2026-08-18: with this, `cargo fuzz init` works, a target ran **32.9 M
executions in 16 s**, and a deliberately unchecked index was caught — exit status
77 plus a crash artifact under `fuzz/artifacts/`.

## ⚠ The THIRD `+toolchain` trap: the MSRV gate

Same root cause as the first, and it bites on a gate you are *required* to run:

```
$ cargo +1.90.0 check --all-targets --all-features
error: no such command: `+1.90.0`
```

Homebrew's cargo does not understand `+toolchain` **at all** — not `+nightly`,
not `+stable`, not a version pin. Go through the shim:

```bash
~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
```

**No `PATH=` prefix is needed here**, unlike the fuzz trap: nothing shells out to
an inner bare `cargo`, so fixing the outer command is enough. The three traps are
one fact with three costs, and the fix differs by one detail each time — which is
exactly why each gets rediscovered.

Measured 2026-08-20, twice in succession by two different agents on SPEC-003, each
losing a loop. The reason it kept happening: **MSRV was the one gate of the ten
with no `just` recipe**, so it was the only one that handed you the raw command
instead of a working one. That is now `just msrv`, and the general lesson is
worth more than the fix — *a gate documented as a raw command is a gate that will
be run wrong.* If you find yourself pasting a bare toolchain command out of a
document, that command belongs in `app.just`.

## ⚠ The FOURTH `+toolchain` trap: clippy's own driver

The three above are about `cargo` not understanding `+toolchain`. This one is
about the *component* being found on PATH after the outer command resolved
correctly — the same shape as the `cargo fuzz` trap, on a different binary.

```
$ ~/.cargo/bin/cargo +stable clippy --version
clippy 0.1.97                                    # Homebrew's, NOT stable's

$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +stable clippy --version
clippy 0.1.98 (88d9e12ae1 2026-08-18)            # what CI actually runs
```

**Why it matters more than the other three.** CI uses
`dtolnay/rust-toolchain@stable`, which **floats**, with `-D warnings`. So every
lint a new clippy release ADDS is an immediate CI failure on code that did not
change — and the local `cargo clippy` (pinned at Homebrew 0.1.97) cannot see it.

Measured 2026-08-22, and **corrected by `PATCH-001`'s independent verify** — the
first version of this paragraph named the drift as the root cause and was wrong:

`main` was red for **17 consecutive runs**, from `1964a7f` (2026-08-20 —
`ship(spec-001)`, the first run that contained the Rust jobs) through `04aaf4b`,
spanning **six** shipped specs. Every verify cycle in that window reported "ten
gates green". They were green *locally*.

⚠ **The drift is the smaller half.** Job-by-job: the **red-proof failed in all
17**; clippy in **14**. The first three reds had clippy green, with the
red-proof's own ANSI-parsing defect the sole cause — so the gate enforcing a
blocking constraint had **never once run successfully in CI**. That is a
different problem from this one and is tracked as
`a-gate-that-fails-mutely-is-a-gate-that-never-ran`.

The fix is a recipe, not a habit: **`just lint-ci`**. Per SPEC-003/FU-8's lesson,
a gate documented as a raw command is a gate that will be run wrong.

## Package manager

`cargo`. **There is no `Cargo.toml` and no `src/` yet** — the first spec of
STAGE-001 creates them. That spec must also: set `edition = "2021"`, set a
`rust-version` (MSRV) **measured from the real dependency set rather than
guessed**, and add the Rust CI jobs (`.github/workflows/ci.yml` currently carries
only the language-agnostic `cost-data` and `decisions-index` gates).

Dependencies are governed by two blocking constraints — `no-copyleft-dependencies`
(permissive only) and `provenance-recorded-per-algorithm`. Adding one is a
decision, not a build step. See AGENTS.md §13, DEC-004 rule 4 as narrowed here.

## Test framework + assertion library

The **built-in** harness — `cargo test`, plain `assert!` / `assert_eq!`. No
external assertion crate is installed or wanted.

```bash
cargo test --all-features                                    # whole suite
cargo test --all-features <name> -- --exact --nocapture      # one test
```

Unit tests go in a `#[cfg(test)] mod tests` at the bottom of the module under
test; integration tests in `tests/<area>.rs`; fuzz targets in
`fuzz/fuzz_targets/<parser>.rs`.

`cargo-insta` **is installed** (see below) but is **not adopted** — snapshot
testing needs its own `DEC-*` first. Do not reach for it unprompted.

## Lint / format quirks

`clippy 0.1.97`, `rustfmt 1.9.0`.

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

The panic-free constraint (`no-panics-on-untrusted-input`) is enforced
mechanically, not by review. This exact lint set is **verified working on clippy
0.1.97** — all five names are valid (no unknown-lint warning) and each *fires as
an error*:

```rust
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
```

Probed in a scratch crate: `v[0]` → *"indexing may panic"*, `v.first().unwrap()`
→ *"used `unwrap()` on an `Option`"*, `a + b` → *"arithmetic operation that can
potentially result in unexpected side-effects"*. All three were hard errors.

**Allow them inside `#[cfg(test)]` modules and in `src/bin/irr.rs`** — those are
not library paths. Do not blanket-allow them anywhere else; reach for `.get()`,
`checked_*` and `try_into()` instead.

## Runtime / target surface — OPEN, do not assume

`DEC-002` (**`status: proposed`**, confidence 0.72) proposes `no_std` + `alloc`
where possible with `std` behind a default-on feature, **no `rayon`**, and
determinism pinned within a `develop_version`. It is explicitly gated on
SPIKE-001 measuring the cost before acceptance.

**So: do not add `rayon`, do not assume `std` is freely available on the
algorithmic path, and do not introduce runtime SIMD dispatch** without checking
DEC-002's status first. If your spec forces the question, that is a signal to
stop and ask, not to decide it in a build cycle.

## Installed dev utilities — do NOT re-add

All already on this machine (`~/.cargo/bin`, so they resolve on `PATH`):

| Tool | Version | Note |
|---|---|---|
| `cargo-fuzz` | 0.13.2 | **needs nightly** — see the trap above |
| `cargo-deny` | 0.19.9 | `cargo deny check licenses` — the permissive-only gate |
| `cargo-insta` | installed | snapshot testing; **not adopted**, needs a DEC |
| `cargo-bloat`, `twiggy` | installed | size analysis (relevant to DEC-002's wasm question) |
| `wasm-bindgen` | installed | ditto |
| `cargo-dist` | installed | release tooling; **not** for this repo during PROJ-001 |

## Oracle tooling (run as TOOLS, never linked)

| Tool | Version | Use |
|---|---|---|
| `dnglab` | 0.7.2 | all three oracle layers — `docs/oracle-contract.md` |
| `exiftool` | 13.55 | metadata cross-check |
| `ssimulacra2` | installed | the develop-layer score (STAGE-003) |
| `magick` (ImageMagick) | installed | authoring PPM inputs for `dnglab makedng` |
| `docker` | installed | the macOS↔Linux byte-identity question (SPIKE-001 Q7) |

⚠ **`dnglab` is LGPL-2.1.** It is *run*, never linked, which imposes nothing.
Never take `rawler`/`rawloader` as a dependency, **including a dev-dependency**.
Reading dnglab's source to solve a problem is a `provenance-recorded-per-algorithm`
violation — run it, don't read it.

## Known gotchas

- **`dnglab makedng` accepts PPM only.** TIFF, PGM, PNG and JPEG are all rejected
  with `Input format is not supported`. PPM is RGB by definition and PGM (the one
  grayscale format) is refused — so **there is no makedng path to a 1-sample
  monochrome fixture.** What it emits is `SamplesPerPixel: 3`,
  `BitsPerSample: 16 16 16`, `Compression: JPEG`; `--linearization` changes none
  of those. Consequence: tier A can exercise the *metadata* oracle but **cannot**
  exercise STAGE-002's 14-bit packed mono unpack. Hand-built headers are the
  route there, not `makedng`. Measured 2026-08-16 — see `docs/oracle-contract.md`.
- **The corpus lives outside the repo.** `~/Pictures/L1021223.DNG` is a real
  `LEICA Q2 MONO` frame (86 MB). Tier-B files are **never committed** —
  `.gitignore` blocks `tests/corpus/tier-b/` and every RAW extension. A single
  `git add -A` after a camera session would put 60 MB blobs in history
  permanently. Check `git status` before staging.
- **The Photos library is TCC-protected.** `~/Pictures/Photos Library.photoslibrary`
  returns `Operation not permitted` to `ls` and `find`. If more camera files are
  needed, they must be exported out of Photos by the maintainer — an agent cannot
  enumerate what is in there, so "I found no more files" is never a safe
  conclusion about that path.
- **Tier-B tests must skip LOUDLY** when the corpus is absent, naming the missing
  file. A silent skip reports green for work it never did — the same defect class
  as an oracle that cannot go red.
- **`irr` is a dev/oracle binary, not a product surface.** `unwrap()` is fine
  there. Never design a library feature around it.

## ⚠ PATH order, not `RUSTUP_TOOLCHAIN`, selects clippy here (PATCH-004, 2026-09-06)

Measured on the maintainer's machine, and it corrects the workaround that gets
passed around:

```
RUSTUP_TOOLCHAIN unset     clippy 0.1.97 @ /opt/homebrew/bin/cargo-clippy
RUSTUP_TOOLCHAIN=stable    clippy 0.1.97 @ /opt/homebrew/bin/cargo-clippy   ← NO CHANGE
PATH=~/.cargo/bin:$PATH    clippy 0.1.98 @ ~/.cargo/bin/cargo-clippy        ← this is what works
```

`cargo` finds a subcommand by looking for `cargo-<name>` **on `PATH`**, so
`RUSTUP_TOOLCHAIN` does not reach it. Homebrew's `cargo-clippy` (rust 1.97.1)
sits earlier than rustup's shim and wins. The default toolchain here is
**nightly, which ships no clippy at all** — so without Homebrew's, bare
`cargo clippy` would simply fail, which is what one reviewer's machine did.

**This is why `just lint-ci` pins both** — `PATH="$HOME/.cargo/bin:$PATH"` *and*
`+stable`. Either alone is insufficient on this machine.

Both lint recipes now print the clippy version and its resolved binary, so you
never have to work this out again — read the first line of their output.
