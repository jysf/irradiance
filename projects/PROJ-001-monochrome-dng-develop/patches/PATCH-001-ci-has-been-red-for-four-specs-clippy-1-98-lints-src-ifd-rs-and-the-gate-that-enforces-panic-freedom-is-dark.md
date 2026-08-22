---
# A PATCH is a lightweight fix to ALREADY-SHIPPED behavior (a bug or UX
# papercut) that adds NO new feature/command and doesn't warrant a full
# spec + stage. See AGENTS.md "Patch lane" and docs/decisions/DEC-003.
#
# Collapsed cycle: patch -> verify -> ship (design+build fused into one
# test-first pass; the INDEPENDENT verify is KEPT). It uses the same task.*
# schema as a spec, so `just validate`, `just cost-audit`, and `just status`
# treat a patch as first-class.

task:
  id: PATCH-001
  type: patch                      # epic | story | task | bug | chore | patch
  cycle: patch                     # patch | verify | ship  (collapsed from a spec's 5)
  blocked: false
  priority: medium
  complexity: S                    # S | M  (an L fix is probably a spec, not a patch)

project:
  id: PROJ-001
  # No `stage:` — a patch attaches to the PROJECT, not a stage.
repo:
  id: irradiance

agents:
  implementer: claude-opus-5  # the patch pass (tier_map.build; DEC-005)
  verifier: claude-opus-5        # independent verify — KEPT (tier_map.verify; a separate session/agent)
  created_at: 2026-08-22

references:
  decisions: []                    # add a DEC only when there's a real decision

# Cost: patch + verify are the metered cycles — `just cost-audit` requires a
# real tokens_total on both for a shipped patch. ship is main-loop (null-with-note).
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# PATCH-001: CI has been red for four specs — a floating clippy under `-D warnings`, and nobody was reading CI

## Problem

**0 of the last 12 CI runs on `main` succeeded.** Red continuously from
`ee5f310` (2026-08-21) through `04aaf4b` — spanning **SPEC-003, SPEC-004,
SPEC-007 and SPEC-008**. Every one of those shipped reporting *"ten gates
green."* They were green **locally**. Nobody read CI.

**Root cause.** `.github/workflows/ci.yml` uses `dtolnay/rust-toolchain@stable`,
which **floats**, with `-D warnings`. Stable moved to `rustc 1.98.0` on
2026-08-18 (the run log records `stable … updated - rustc 1.98.0 (from rustc
1.97.1)`), and 1.98's clippy added `chunks_exact_to_as_chunks`. It fires on
**seven pre-existing sites** that no spec had touched:

| file | sites |
|---|---|
| `src/ifd.rs` | `:772` `ENTRY_SIZE`, `:876` `(2)`, `:884` `(4)`, `:895` `(8)` |
| `tests/support/corpus.rs` | `:455` `(BLOCK)`, `:486` `chunks_exact_mut(4)`, `:496` `(4)` |

⚠ The CI log showed only the first four — it stops at `could not compile
irradiance (lib)`. The other three were behind that failure and only appeared
once the lib was fixed.

**Why this is worse than a red badge.** Two of the failing jobs are
`clippy -D warnings` and **`lint policy red-proof`** — the latter is
`no-panics-on-untrusted-input`'s mechanical enforcement (DEC-009).
`constraints.yaml` says of that pair: *"Do not delete either job without
replacing its guarantee."* **A job that has failed for four specs is
functionally deleted**, and worse: it now fails for an *unrelated* reason, so a
genuine red would be indistinguishable from the noise.

**It was foreseen and not acted on.** `SPEC-006`'s handback recorded on
2026-08-18 that *"`toolchain-brief.md`'s `+stable = 1.97.0` has drifted to
1.98.0 on this host."* It was filed as a minor note. That note is this outage.

## Fix

**Three parts. The third is the one that stops it recurring.**

1. **All seven sites → `as_chunks::<N>()` / `as_chunks_mut::<N>()`.** ⚠ MSRV
   verified first, not assumed: compiled and ran a probe under
   `rustc +1.90.0` (the pinned MSRV) before touching anything.
   This is a **net reduction in fallible code on a parse path**, which is why it
   was preferred to pinning or `#[allow]`: `as_chunks` makes the length a
   type-level fact, so five `try_into()?` / `get(..)?` pairs that could never
   fail are **gone** rather than merely unreachable. In `sha256` it also removes
   two `copy_from_slice` into scratch arrays.
2. **`just lint-ci`** — clippy as CI sees it, plus its line in AGENTS.md §6
   (recipe↔block correspondence is SPEC-001 AC8) and the trap in
   `guidance/toolchain-brief.md`.
3. ⚠ **The FOURTH `+toolchain` trap, measured.** `~/.cargo/bin/cargo +stable
   clippy --version` reports **0.1.97** — the outer command resolves through the
   shim, but `clippy-driver` is then found on `PATH`, where Homebrew's wins.
   With `PATH="$HOME/.cargo/bin:$PATH"` it reports **0.1.98**. Same shape as the
   `cargo fuzz` trap on a different binary. **This is why nobody could have
   caught the drift locally even if they had tried** — the obvious command
   silently gives you the wrong clippy.

**Deliberately NOT done: pinning CI's toolchain.** Floating is what surfaces
drift; pinning would have made this quiet instead of fixed, and would freeze the
panic-free lint policy at an old clippy. The recurrence question — `-D warnings`
on a floating toolchain means *every* future clippy release can break CI on
unchanged code — is real and is **not** settled here. Filed as a signal.

## Failing Tests

The gate is the test. Both directions run with the CI-parity clippy:

- **`just lint-ci` on the unpatched tree** → **9 lint hits** (the negative
  control — measured by reverting `src/ifd.rs` to its pre-patch bytes, confirmed
  changed by `diff`, then restored byte-identical).
- **`just lint-ci` on the patched tree** → **exit 0.**
- `cargo test --all-features` → **66 passed**, summed across targets. The
  `sha256` rewrite is covered by the published NIST vectors and by
  `sha256_streaming_matches_one_shot`, which is precisely the test that would
  catch a mis-chunked block boundary.
- Fuzz, per §12 bar 2 — a parse path changed: **12,633,398 runs / 61 s, zero
  artifacts.**

## Verification (independent — KEPT)

Patch lane keeps the independent verify (DEC-003). See `## Patch Completion`.

## Defect-catch stage

`escaped` — it reached `main` and stayed there for four shipped specs. Not
caught by design, build, verify or ship, because **every one of those checked
the local gates and none checked CI.** The behavioural pre-flight §15 check 8
asks for exists and was not applied to the CI surface itself.

