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

## ⚠ A SECOND defect, found only because the first was fixed

Fixing the lint turned `clippy -D warnings` green — and the **`lint policy
red-proof` job still failed**, now for an unrelated reason that had been latent
the whole time.

`scripts/lint-red-proof.sh`'s assertion 4 greps its captured clippy log for
`'--> src/lib\.rs:[0-9]+'`. **CI's clippy colourises even when redirected to a
file.** The real bytes are:

```
\e[1m\e[94m--> \e[0msrc/lib.rs:66
```

— a reset sequence sits **between** `-->` and the path, so the grep matches
nothing, exits 1, and under `set -o pipefail` + `set -e` **kills the script
before its own `die` can print.** Exit 1, no message.

**A proof that dies without a message is indistinguishable from a proof that
never ran** — which is the exact defect class this file exists to prevent. It
was unreachable until now only because the job had been failing *earlier*, at
the control run.

It is also the general form of [[attribute-text-inside-doc-comments]] arriving a
fourth way: *text matching on tool output finds what the tool decided to
render*, and a zero-match must be **asserted**, never allowed to become control
flow.

**Fix, two parts:**
1. `--color never` on both clippy invocations in `run_clippy`, so the log is
   parseable deterministically on any host regardless of CI's colour behaviour.
2. `|| true` on the leading grep, so a zero-match **flows to the assertion** and
   the `die` explains itself instead of the script aborting mutely.

**Red-proofed, with a control** — CI's condition reproduced locally via
`CARGO_TERM_COLOR=always`:

| | exit | explanatory output |
|---|---|---|
| **old** script + forced colour | **1** | **none** — CI's silent death, reproduced |
| **new** script + forced colour | **0** | full success line |
| new script, local (0.1.97) and CI-parity (0.1.98), no forced colour | 0 | full success line |

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

Patch lane keeps the independent verify (DEC-003). Brief for the reviewer below.
Record the verdict in `## Patch Completion`.

**⚠ The orchestrator wrote both fixes, both red-proofs and this record.** That is
why this verify is kept. `SPEC-005` round 2 is the precedent: the last two
artefacts written by this author and self-graded were each wrong — `DEC-013` on
three counts, and its replacement doc comment on a fourth.

**State:** `main` at `96be26c`, **all nine CI jobs green** (run `32596678286`) —
the first success in at least 13 runs. Confirm that yourself; do not take it from
here.

### FOUR claims you must not inherit

**1. `as_chunks` is MSRV-safe.** The choice to fix rather than pin rests entirely
on `as_chunks::<N>()` compiling on the pinned **1.90.0**. Reproduce:
`~/.cargo/bin/rustc +1.90.0 --edition 2021` on a probe, then `just msrv`. If it
does not hold, the whole fix is wrong and pinning was the answer.

**2. Seven sites was all of them.** `grep -rn 'chunks_exact' --include='*.rs' src/
tests/ examples/ fuzz/`. ⚠ The CI log originally showed only **four** — the other
three hid behind `could not compile irradiance (lib)`. Satisfy yourself nothing
is hiding behind *this* green the same way, including in targets CI builds but
`just lint` might not.

**3. The refactor is behaviour-preserving. This is the one that matters most.**
It touched a **parse path** and a **hash**. Tests passing is necessary, not
sufficient — convince yourself per site:
- `src/ifd.rs` `TYPE_RATIONAL`: `chunks_exact(8)` + two `get(..)?.try_into()?`
  became `as_chunks::<8>()` + `let [n0,n1,n2,n3,d0,d1,d2,d3] = *chunk;`. Is the
  **byte order of the destructure** identical to the two 4-byte slices it
  replaced?
- `tests/support/corpus.rs:455`: `chunks_exact(BLOCK)` + `.remainder()` became
  `as_chunks::<BLOCK>()`'s `(blocks, rest)`. **Are `rest` and `remainder()` the
  same bytes in every case**, including an input that is an exact multiple of
  `BLOCK`, and one shorter than `BLOCK`? A streaming hash that drops or
  double-counts a tail byte is a silent wrong answer, and `DEC-003` pins corpus
  files by `sha256` — a wrong hash makes every corpus file "corrupt".
- Confirm `sha256_streaming_matches_one_shot` and the **published NIST vectors**
  actually run (per-target `-- --list`, sum across targets — a zero-match
  `cargo test <name>` exits 0) and that they would catch a mis-chunked boundary.
  The existing comment claims a split at 4999; check that is still exercised.

**4. The ANSI diagnosis.** Reproduce CI's condition locally:
`CARGO_TERM_COLOR=always bash scripts/lint-red-proof.sh`. On the **pre-fix**
script (`git show 96be26c^:scripts/lint-red-proof.sh`) that must exit **1 with no
explanatory output**; on the current one, exit **0**. If the pre-fix script does
*not* die, the diagnosis is wrong and the real cause is still out there.

### The check I most want, and did not do

**Does `|| true` weaken assertion 4?** It was added so a zero-match `grep` reaches
the `die` instead of aborting mutely — but `|| true` is exactly how a real failure
gets swallowed. **Force a genuine zero-match** (e.g. point `INJ_FIRST`/`INJ_LAST`
outside the injected range, or inject nothing) and confirm the script now **fails
loudly with its own message** rather than passing. If a real zero-match can now
reach a green, that is a **ship-blocker** and worse than the bug it fixed.

### Also

- **Ten gates plus `just lint-ci`**, re-run by you, summed across all targets.
- **`just lint-ci` must be the recipe it claims to be** — check the `PATH=`
  prefix is present and that removing it drops you to 0.1.97. That is the whole
  point of the recipe.
- **Fuzz** — a parse path moved (§12 bar 2). The patch claims 12,633,398 runs /
  61 s / zero artifacts.
- **Scope:** anything in `96be26c^^..96be26c` that is not these two fixes or their
  records is scope creep. Call it.
- ⚠ **Do not re-litigate the pin-vs-fix decision** as a finding unless you think
  it is *wrong*. The standing trade-off is deliberately filed as risk
  `floating-toolchain-plus-deny-warnings` for the project close, with three
  options; disagreeing with *that deferral* is fair game.

**Findings:** `SB-N` / `FU-N`, numbered for **PATCH-001** from 1, each with which
of §15's four dispositions you think it wants. Verdict: ✅ APPROVED (with SHA) /
⚠ PUNCH LIST / ❌ REJECTED.

**Cost:** record a real `tokens_total` **deduped by `message.id`** and say you
deduped; compute `estimated_usd` per-component at the rates for the model that
**actually ran** (`message.model`, not `tier_map` — it is 1 for 4). Capture as
late as you can: the "floor" convention measured ~17% low on `SPEC-005`.

## Defect-catch stage

`escaped` — it reached `main` and stayed there for four shipped specs. Not
caught by design, build, verify or ship, because **every one of those checked
the local gates and none checked CI.** The behavioural pre-flight §15 check 8
asks for exists and was not applied to the CI surface itself.

