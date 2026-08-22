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
  cycle: verify                    # patch | verify | ship  (collapsed from a spec's 5)
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
  sessions:
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 10000000
      estimated_usd: 24.90
      duration_minutes: 55
      recorded_at: 2026-08-22
      notes: "Independent verify of PATCH-001 at c85b8bd on main, IRRADIANCE_CORPUS_DIR set (7/7 corpus files present, no test skipped). Verdict PUNCH LIST: 2 ship-blockers, 4 follow-ups. SB-1 is the check the brief asked for and did not do: the `|| true` added to assertion 4 does NOT stop the silent death. It guards the FIRST grep, but a zero-match there emits nothing, so the SECOND grep (`grep -oE '[0-9]+$'`, scripts/lint-red-proof.sh:297) also zero-matches, exits 1, and under `set -o pipefail` + `set -e` aborts the command substitution BEFORE the die. Proven on the real script, not a model: HEAD with only the `--color never` half reverted -- i.e. `|| true` as the sole guard, the exact configuration the commit credits -- under CARGO_TERM_COLOR=always exits 1 with its last narration a green checkmark and no die, byte-for-byte the pre-fix behaviour. The brief's suggested test (point INJ_FIRST/INJ_LAST outside the range) would have PASSED while the defect stayed: that path reaches the die loudly (verified: 'only 0 distinct diagnostic span(s)'), because grep matches and awk filters to empty. Only a genuine zero-match of the leading grep exposes it. Fix verified both directions: `| { grep -oE '[0-9]+$' || true; }`. SB-2: the record's root cause is INVERTED. The ANSI defect was never latent -- at 1964a7f (2026-08-20T23:33, the first CI run containing the Rust jobs) the log shows 'clippy 0.1.98', then 'control: unmutated copy is clean', then exit 1 with nothing else. The control PASSED and the script died at assertion 4 on day one. Job-by-job across the streak: red-proof failed in ALL 17 runs; clippy failed in 14 (c114339..04aaf4b); the first three runs had clippy GREEN and the ANSI defect as the sole cause of red. The dark gate is the older and more serious defect and the drift does not explain it -- which matters because signals.yaml's evidence field encodes the drift as root cause and that signal is what the project close will act on. FU-1: outage understated in six documents -- 17 consecutive failures 1964a7f..f2d0513 spanning SIX specs (SPEC-001 ship, 006, 003, 004, 007, 008), not 12 runs / four specs; streak starts at ship(spec-001), not ee5f310. FU-2: the '9 lint hits' negative control does not reproduce as attributed -- reverting src/ifd.rs alone gives 4, with the three corpus.rs sites hidden behind 'could not compile irradiance (lib)'; 9 is the ifd-fixed/corpus-reverted configuration (3 targets x 3 sites). FU-3: the zero-match-as-control-flow class got no signals.yaml entry though the record names it as a fourth instance of attribute-text-inside-doc-comments. FU-4: fuzz/ is outside every -D warnings clippy gate (clean today under 0.1.98). THE FOUR CLAIMS, NOT INHERITED. (1) MSRV: probe using as_chunks/as_chunks_mut, the 8-byte destructure and a rest-vs-remainder differential compiled and ran under rustc +1.90.0 --edition 2021; just msrv green after touching the changed files, because the first run finished in 0.26s off a warm cache. (2) Seven sites: whole-tree grep returns one hit today and it is a comment; f2d0513^ returns exactly the seven claimed; a COLD run in a fresh CARGO_TARGET_DIR exited 0 with artifacts proving all targets linted (lib x2, bin irr x2, both integration tests, both examples); no [features] block; fuzz graph checked separately under 0.1.98. (3) Behaviour-preserving: rest == remainder() asserted byte-equal at n=0,1,63,64,65,128,191; destructure byte order confirmed identical to get(0..4)/get(4..8); FOUR adversarial mutations all caught -- dropped sha256 tail (all three sha256 tests red, NIST included), removed !data.is_empty() guard (red PRECISELY at split 4999, verifying corpus.rs:453's own comment), swapped num/den (2 rational tests red), reversed the SHORT chunk (4+ red); tree restored byte-identical via shasum -c after each; per-target -- --list confirms both sha256 tests exist in exactly one target (no zero-match) and corpus_files_match_their_pinned_sha256 takes ~11.8s, so it is hashing, not skipping. (4) ANSI mechanism confirmed exactly -- real bytes are \e[1m\e[94m--> \e[0msrc/lib.rs:79; pre-fix + forced colour exits 1 with no die, current exits 0, and green under true CI parity (RUSTUP_TOOLCHAIN=stable, clippy 0.1.98) with and without colour. TEN GATES PLUS just lint-ci, all re-run by me: build 0; test 66 passed (45+0+9+12+0 summed across targets, 0 failed 0 ignored); lint 0; typecheck 0; deny 0; deny-fuzz 0; msrv 0 (forced recompile vs exactly 1.90.0); lint-red-proof 0; lint-no-allow 0; fuzz 11,325,015 runs / 61s with fuzz/artifacts/ifd/ empty and git status clean after; lint-ci 0 warm AND cold. Plus just validate (10 artifacts), just cost-audit (passes), just decisions-audit --changed (no match; DEC-009 does scope lint-red-proof.sh -- re-read against 96be26c, negative control untouched, no drift). lint-ci PROVEN LOAD-BEARING BEHAVIOURALLY, not just by --version: on the identical unpatched tree the bare shim command exits 0 with ZERO hits (a silent false green) and the PATH-prefixed one exits 101 with the lint firing. CI confirmed here, not taken from the record: run 32596678286 at 96be26c, all 9 jobs success; c85b8bd also green; first success after 17 failures. Scope clean, no creep; the pin-vs-fix deferral not re-litigated. tokens_total is a transcript sum DEDUPED BY message.id from this session's own JSONL (176 usage objects -> 78 distinct ids, a 2.26x inflation if not deduped): input 156 + output 66,161 + cache_read 9,202,340 + cache_write 156,987 (all on the 1-hour ephemeral tier, 5-minute tier 0) = 9,425,644, 97.6% cache-read. Rounded UP to 10,000,000 to cover the turns spent finishing this note and committing -- captured as late as possible because the floor convention measured ~17% low on SPEC-005. estimated_usd computed PER-COMPONENT at published Opus rates ($15/M input, $75/M output, $30/M cache-write-1h, $1.50/M cache-read) on the measured 9,425,644 figure ($23.48), scaled to the rounded total ($24.90); model is claude-opus-5 read from message.model in the transcript, NOT from tier_map. Not a harness-reported figure -- flagged so it is not mistaken for measured."
  totals:
    tokens_total: 10000000
    estimated_usd: 24.90
    session_count: 1
---

# PATCH-001: CI has been red for four specs — a floating clippy under `-D warnings`, and nobody was reading CI

## Problem

> ⚠ **This section was WRONG when first written and is corrected here.** The
> independent verify (`SB-2`) measured the history job-by-job and found the
> causality inverted. Both versions are kept: the original claim is struck
> through below, because *how* it was wrong is the lesson.

**~~0 of the last 12 CI runs on `main` succeeded, red from `ee5f310`
(2026-08-21) across four shipped specs.~~**

**17 consecutive CI runs on `main` failed** — from `1964a7f` (2026-08-20,
`ship(spec-001)`, **the first run that ever contained the Rust jobs**) through
`04aaf4b`, spanning **six** shipped specs: SPEC-001, 006, 003, 004, 007, 008.
Every one shipped reporting *"ten gates green."* They were green **locally**.
Nobody read CI.

⚠ **And the toolchain drift is the smaller, later half.** Job-by-job: the
**red-proof failed in all 17 runs; clippy in 14.** The first three reds had
clippy **green**, with the red-proof's own ANSI-parsing defect the sole cause.
So the gate that mechanically enforces a **blocking constraint** had *never once
run successfully in CI* — it was born dark. The drift did not cause that and does
not explain it.

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

- **`just lint-ci` on the unpatched tree** → red (the negative control —
  measured by reverting `src/ifd.rs` to its pre-patch bytes, confirmed changed by
  `diff`, then restored byte-identical).
  ⚠ **Corrected:** an earlier draft of this line said *"9 lint hits"* and
  attached that figure to the wrong scenario. Reverting `src/ifd.rs` **alone**
  yields **4**, with `tests/support/corpus.rs`'s three hidden behind the lib
  failure — the same masking that hid them from the original CI log, faithfully
  reproduced. The `9` was a count of matching *log lines* across a
  both-files-unpatched tree, not of sites. The direction was right and the
  number was not; per `measurement-over-generalised`, the number is the part
  that has to be exact.
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

## Findings — dispositioned (AGENTS.md §15)

The independent verify returned ⚠ PUNCH LIST with two ship-blockers and four
follow-ups. Both `SB`s were the orchestrator's, and both are **confirmed
independently before being accepted** — the shell-level repro for `SB-1`, the
22-run job-by-job history for `SB-2`.

| id | finding | disposition |
|---|---|---|
| `SB-1` | `\|\| true` on the leading grep does **not** stop the silent death — a zero-match emits nothing, so the *second* grep zero-matches, exits 1, and pipefail aborts the assignment before the `die` | `fixed` — every stage of the pipeline is now guarded. **Red-proofed with a genuine leading-grep zero-match**: the script now exits 1 **with** `ERROR: … only 0 distinct diagnostic span(s) point inside the injected block`. Fails closed *and* audibly |
| `SB-2` | the root cause is inverted — the ANSI defect was never latent; the red-proof was dark from the first run that ever contained it | `fixed` — corrected in four documents, and the conflated signal **split in two**: `floating-toolchain-plus-deny-warnings` keeps the drift half (14 of 17 runs), and the new lesson `a-gate-that-fails-mutely-is-a-gate-that-never-ran` takes the older half. The close would otherwise have fixed the toolchain and left the mute gate |
| `FU-1` | the outage is understated in six documents — 17 runs / six specs, not 12 / four | `fixed` — corrected in `AGENTS.md`, `guidance/toolchain-brief.md`, `guidance/signals.yaml` and this record. The original claim is struck through rather than deleted |
| `FU-2` | the "9 lint hits" figure is attached to the wrong scenario | `fixed` — corrected above; reverting `src/ifd.rs` alone gives **4**, the other three masked behind the lib failure |
| `FU-3` | the zero-match class got no `signals.yaml` entry despite being named a fourth instance | `signal: a-gate-that-fails-mutely-is-a-gate-that-never-ran` — created, `N=4`, **past its bar**, with the codification text and both traps written in |
| `FU-4` | `fuzz/` sits outside every `-D warnings` gate | `spec: SPEC-011` — **confirmed**: root `cargo clippy --all-targets` never mentions `irradiance-fuzz` (0 hits), because `DEC-011` deliberately keeps it out of the library's graph. Exactly the shape `deny` / `deny-fuzz` already has, and it needs the same second invocation — plus its own red-proof, which is why it is a spec and not a line here |

**2 ship-blockers + 4 follow-ups · 4 `fixed` · 1 `signal` · 1 → `SPEC-011`.**

## Defect-catch stage

`escaped` — it reached `main` and stayed there for four shipped specs. Not
caught by design, build, verify or ship, because **every one of those checked
the local gates and none checked CI.** The behavioural pre-flight §15 check 8
asks for exists and was not applied to the CI surface itself.


---

## Patch Completion

**Verdict: ⚠ PUNCH LIST** — verified independently at `c85b8bd`, on `main`, with
`IRRADIANCE_CORPUS_DIR` set (7/7 corpus files present; no test skipped).

Both fixes are substantively correct and CI is genuinely green. Two findings are
ship-blocking: **the `|| true` half of the second fix does not do what it claims**,
and **the record's root-cause narrative is inverted** — the ANSI defect was never
latent, and it, not the clippy drift, is what kept the blocking-constraint gate
dark for its entire existence.

### The four claims, checked and not inherited

**1. `as_chunks` is MSRV-safe — HOLDS.** A probe using `as_chunks::<N>()`,
`as_chunks_mut::<N>()`, the `&[u8; 8]` destructure and a `rest`-vs-`remainder()`
differential compiled and ran under `~/.cargo/bin/rustc +1.90.0 --edition 2021`
(exit 0, `PROBE OK`). `just msrv` green against exactly `rustc 1.90.0
(1159e78c4)` — re-run after `touch`ing the changed files, because the first run
finished in 0.26 s off a warm cache and a cached green is not a green.

**2. Seven sites was all of them — HOLDS.** `grep -rn 'chunks_exact' --include='*.rs'`
over the *whole* tree (not just `src/ tests/ examples/ fuzz/`) returns exactly one
hit today, and it is a comment. At `f2d0513^` it returned exactly the seven
claimed, at the claimed lines. Nothing is hiding behind this green:
- A **cold** CI-parity run in a fresh `CARGO_TARGET_DIR` exited 0 with zero
  warnings, and the artifacts confirm every target was actually linted — lib
  (×2), bin `irr` (×2), tests `corpus_manifest` + `ifd_reader`, examples
  `corpus_status` + `fuzz_seeds`.
- `Cargo.toml` has **no `[features]` block**, so `--all-features` is not hiding a
  combination.
- The **fuzz graph** is outside every `-D warnings` gate (CI's `clippy` job and
  `just lint-ci` both run the root manifest only). Checked it anyway under
  clippy 0.1.98: clean. See `FU-4`.

**3. The refactor is behaviour-preserving — HOLDS, and the tests have teeth.**
- `rest` ≡ `remainder()` was measured, not reasoned: asserted byte-equal at
  n = 0, 1, 63, 64, 65, 128, 191 — covering the exact-multiple and
  shorter-than-`BLOCK` cases the brief names.
- The `TYPE_RATIONAL` destructure is byte-order-identical to the two slices it
  replaced (`[n0..n3]` ← `get(0..4)`, `[d0..d3]` ← `get(4..8)`), confirmed on the
  probe. Every removed `try_into()?` / `get(..)?` was unreachable by construction,
  so no error path was lost.
- **Four adversarial mutations, all caught**: dropping the sha256 tail → all three
  sha256 tests red (NIST vectors included); removing the `!data.is_empty()` guard
  → `sha256_streaming_matches_one_shot` red **precisely at split 4999**, which
  verifies `corpus.rs:453`'s own comment rather than taking it; swapping num/den
  in the destructure → 2 rational tests red; reversing the `SHORT` chunk → 4+
  tests red. Tree restored byte-identical after each (`shasum -c`).
- The tests **run**: per-target `-- --list` puts `sha256_matches_published_vectors`
  and `sha256_streaming_matches_one_shot` in exactly one target each (no
  zero-match), and `corpus_files_match_their_pinned_sha256` takes ~11.8 s — it is
  hashing real corpus files, not skipping.

**4. The ANSI diagnosis — the MECHANISM holds, the HISTORY does not.**
Reproduced exactly: the real bytes are `\e[1m\e[94m--> \e[0msrc/lib.rs:79`, a
reset between `-->` and the path. Pre-fix script (`96be26c^`) under
`CARGO_TERM_COLOR=always` → **exit 1, last narration a green `✓`, no `die`, no
`✗`**. Current script → exit 0, full success line. Also green under true CI parity
(`RUSTUP_TOOLCHAIN=stable`, clippy 0.1.98) both with and without forced colour.
But the claim that it "was unreachable until now" is false — see `SB-2`.

### Findings

| id | label | finding | suggested disposition |
|---|---|---|---|
| `SB-1` | ship-blocker | `\|\| true` does not stop the silent death — the *second* grep is unguarded | `fixed` |
| `SB-2` | ship-blocker | Root cause is inverted: the ANSI defect was never latent, and it alone darkened the gate for the first 3 runs | `fixed` |
| `FU-1` | follow-up | The outage is understated in six documents: 17 runs / six specs, not 12 / four | `fixed` |
| `FU-2` | follow-up | The "9 lint hits" negative control is attached to the wrong scenario | `fixed` |
| `FU-3` | follow-up | The zero-match-as-control-flow class got no signal entry, though the record names it as a fourth instance | `signal: attribute-text-inside-doc-comments` |
| `FU-4` | follow-up | `fuzz/` sits outside every `-D warnings` clippy gate | `signal: floating-toolchain-plus-deny-warnings` |

---

#### `SB-1` — the check the orchestrator asked for, and it fails

**`|| true` does not make a zero-match reach the `die`. The script still dies
mutely — the exact defect `96be26c` was written to remove.**

The brief anticipated the risk as *"a real zero-match can now reach a green."* It
cannot; the gate still fails closed. The actual behaviour is different and, for
this repo, worse: **it fails closed and says nothing.**

`assert_lints_fired`'s successor pipeline is:

```sh
IN_RANGE="$( { grep -oE -- '--> src/lib\.rs:[0-9]+' "$CLIPPY_LOG" || true; } \
    | grep -oE '[0-9]+$' \                       # <-- UNGUARDED
    | awk ... | sort -un | wc -l | tr -d '[:space:]')"
```

`|| true` rescues the *first* grep. But when the first grep matches nothing it
emits nothing, so the **second** grep also zero-matches and exits 1. Under
`set -o pipefail` that is the pipeline's status, and `set -e` aborts the
assignment — **before the `die` on the next line can run.**

Measured, on the real script, not a model of it. Taking the current `HEAD`
script and reverting *only* the `--color never` half — so `|| true` is the sole
guard, exactly the configuration the commit message credits with making the
`die` explain itself — under `CARGO_TERM_COLOR=always`:

| script | exit | explanatory output |
|---|---|---|
| `96be26c^` (neither fix) | 1 | none — last line is `✓ control:` |
| `HEAD` minus `--color never`, **`\|\| true` present** | **1** | **none — last line is `✓ control:`** |
| `HEAD` (both fixes) | 0 | full success line |

The middle row is the finding: `|| true` changes nothing.

Which zero-match paths are safe is worth stating precisely, because one of them
*does* work and it is the one that makes the bug easy to miss:

- **grep matches, `awk` filters to empty** (the brief's "point `INJ_FIRST`/`INJ_LAST`
  outside the range" suggestion) → `IN_RANGE=0` → **`die` fires loudly.** Verified:
  `ERROR: clippy failed and named the lints, but only 0 distinct diagnostic
  span(s) point inside the injected block (src/lib.rs lines 99000-99999)`.
- **grep matches nothing at all** (the ANSI case, or any future change to how
  clippy renders a span) → **silent death, exit 1, no message.**

So the suggested test would have *passed* while the defect stayed. Only forcing a
genuine zero-match of the leading grep exposes it.

`--color never` removes today's trigger, which is why every gate is green. It does
not remove the defect: any other cause of a zero-match reproduces the silent
death identically, and the comment now sitting above that line asserts a
guarantee the code does not provide.

**Fix — one guard, verified both directions:**

```sh
    | { grep -oE '[0-9]+$' || true; } \
```

Zero-match → `IN_RANGE=0` → `die` fires with its own message. Matching log →
`IN_RANGE=4` → passes. `scripts/lint-red-proof.sh:297`.

`grep` at `:278` is already guarded (`|| missing=...`) and `:326` guards its whole
pipeline; `:297` is the only remaining hole in the file.

---

#### `SB-2` — the ANSI defect was never latent, and the root cause is inverted

The record says the ANSI failure *"had been latent the whole time"* and *"was
unreachable until now only because the job had been failing **earlier**, at the
control run."* The CI history says otherwise.

`gh run view --job 96617122824 --log` for `1964a7f` — **2026-08-20T23:33, the very
first run that contained the Rust jobs**:

```
• clippy is present: clippy 0.1.98 (88d9e12ae1 2026-08-18)
• control run: the UNMUTATED copy, exact CI invocation — this MUST pass:
✓ control: unmutated copy is clean (clippy exit 0).       <-- the control PASSED
• mutation run: the same invocation on the injected copy — this MUST fail:
##[error]Process completed with exit code 1.              <-- and nothing else
```

The control did not fail. The script reached assertion 4 and died there, mutely,
on day one. Job-by-job across the whole streak:

| runs | `clippy -D warnings` | `lint policy red-proof` |
|---|---|---|
| `1964a7f`, `93009b2`, `dd4eb42` | **success** | **failure** |
| `c114339` … `04aaf4b` (14 runs) | failure | failure |
| `f2d0513` (fix 1 landed) | success | **failure** |

**The red-proof failed in all 17 runs. Clippy failed in 14.** For the first three
there was no clippy lint at all — `chunks_exact` had not yet been written — and
the ANSI defect was the *sole* cause of red.

This inverts the record's story. The ANSI defect is not a second thing found
downstream of the first; it is the **older and more serious** of the two, and it
is the one that actually answers the patch's own title question — *the gate that
enforces panic freedom was dark from the moment it was switched on, and it never
once ran to completion.* The clippy drift is real, is correctly diagnosed, and
explains the `clippy` job; it does not explain the dark gate.

This matters beyond tidiness: `## Defect-catch stage`, the "Foreseen and not
acted on" paragraph, and `guidance/signals.yaml`'s `evidence:` field all encode
the drift as the root cause, and that signal is what the project close will act
on. As written, the close would weigh options for the floating toolchain and
never see that the blocking-constraint gate failed for an unrelated reason that
no toolchain policy would have caught.

---

#### `FU-1` — the outage is understated, in six places

Measured with `gh run list --branch main --limit 60`:

- **17 consecutive failures**, `1964a7f` (2026-08-20T23:33) → `f2d0513`
  (2026-08-22T20:21). Last green before: `a84efc1`, 2026-08-20T23:09.
- The streak begins at **`1964a7f` = `ship(spec-001)`**, not `ee5f310`, and spans
  **six** specs — SPEC-001's ship, SPEC-006, SPEC-003, SPEC-004, SPEC-007,
  SPEC-008 — not four.

"0 of the last 12" and "the first success in at least 13 runs" are both *true*
(they are subsets), but "red continuously from `ee5f310` through `04aaf4b`" is
wrong at the start boundary. The wrong span appears in the patch record, the
`f2d0513` commit message, `AGENTS.md` §6, `app.just`, `guidance/toolchain-brief.md`
and `guidance/signals.yaml`.

#### `FU-2` — the negative control's number doesn't reproduce as attributed

The record: *"`just lint-ci` on the unpatched tree → 9 lint hits (measured by
reverting `src/ifd.rs`)."* Reverting `src/ifd.rs` alone gives **4** rendered
errors (`(lib)` and `(lib test)` each report "4 previous errors"), and the three
`corpus.rs` sites stay **hidden behind `could not compile irradiance (lib)`** —
the same masking the record describes for the original CI log, faithfully
reproduced.

**9** is the count from a *different* configuration: `ifd.rs` fixed and
`corpus.rs` reverted, where `tests/support/corpus.rs` compiles into three targets
(`corpus_manifest`, `ifd_reader`, and example `corpus-status`) × 3 sites. The
number is real; it is attached to the wrong scenario, and as written it implies
the unpatched tree surfaces all seven at once, which is the one thing this patch
proves it does not.

#### `FU-3` — the class was named and then not filed

The record identifies the ANSI failure as
`[[attribute-text-inside-doc-comments]]` *"arriving a fourth way"* and states the
general rule — *a zero-match must be asserted, never allowed to become control
flow.* `96be26c` did not touch `guidance/signals.yaml`; that signal's
`last_touched` is still `2026-08-18` and its `evidence` still reads N=5 with no
mention of this. Per §15 a class-level recurrence disposes as `signal:`. `SB-1`
is the same class a fifth time, in the fix itself.

#### `FU-4` — `fuzz/` is outside every `-D warnings` gate

CI's `clippy` job and `just lint-ci` both lint the root manifest only; nothing
compiles `fuzz/` under `-D warnings` on any toolchain. It is clean today
(verified under clippy 0.1.98), so nothing is hiding — but the drift that caused
this outage would be invisible there, and options 2 and 3 in
`floating-toolchain-plus-deny-warnings` should say whether the fuzz graph is in
scope.

### Gates — ten plus `just lint-ci`, all re-run by me

| gate | result |
|---|---|
| `just build` | exit 0 |
| `just test` | **66 passed** — 45 lib + 0 `irr` + 9 `corpus_manifest` + 12 `ifd_reader` + 0 doc, summed across targets; 0 failed, 0 ignored |
| `just lint` (clippy + fmt) | exit 0 |
| `just typecheck` | exit 0 |
| `just deny` | exit 0 |
| `just deny-fuzz` | exit 0 |
| `just msrv` | exit 0, forced recompile against exactly 1.90.0 |
| `just lint-red-proof` | exit 0 — control clean → injection rejected → five lints fired → still fire without `-D warnings` |
| `just lint-no-allow` | exit 0 |
| `just fuzz` | **11,325,015 runs / 61 s, `fuzz/artifacts/ifd/` empty**, `git status` clean after |
| `just lint-ci` | exit 0 warm, and exit 0 **cold** in a fresh target dir |

Plus `just validate` (10 artifacts, valid front-matter), `just cost-audit`
(passes), `just decisions-audit --changed` (no active decision's `affected_scope`
matches). `DEC-009`'s scope *does* list `scripts/lint-red-proof.sh`; re-read it
against `96be26c` — the negative control is untouched, no drift.

Fuzz run count differs from the record's 12,633,398 (run-to-run variance on a
time-boxed target); the material claim, **zero artifacts**, reproduces.

### `just lint-ci` is the recipe it claims to be — proven behaviourally

The `PATH=` prefix is present, and `--version` reports 0.1.97 without it and
0.1.98 with it, as recorded. Stronger, on the **identical unpatched tree**:

| command | exit | `chunks_exact` hits |
|---|---|---|
| `~/.cargo/bin/cargo +stable clippy --all-targets --all-features -- -D warnings` | **0** | **0** |
| `PATH="$HOME/.cargo/bin:$PATH"` + the same | **101** | 5 |

The obvious command hands you a **silent false green** on a tree that fails CI.
The claim that nobody could have caught the drift locally is verified, not
inherited. `app.just`'s `lint-ci` body is byte-identical to the `AGENTS.md` §6
block (SPEC-001 AC8 holds for this change).

### CI, confirmed here rather than taken from the record

Run `32596678286` at `96be26c`: **all 9 jobs success** (`fmt`, `clippy`,
`red-proof`, `licenses`, `test`, `licenses-fuzz`, `no-allow`, `cost-data`,
`msrv`). `c85b8bd` also green. The first success after 17 consecutive failures.

### Scope

`96be26c^^..96be26c` touches `src/ifd.rs`, `tests/support/corpus.rs`,
`scripts/lint-red-proof.sh` (the two fixes); `app.just` + `AGENTS.md` §6 (the
recipe, required together by AC8); `guidance/toolchain-brief.md` (the fourth
trap); `guidance/signals.yaml` (the deferred risk); and the patch record.
**No scope creep.** The pin-vs-fix deferral is not re-litigated — the trade-off is
correctly filed with three options, and option 2 does keep both properties.

Minor, not numbered: the `## Verification` section cites bare `DEC-003` for the
patch lane. That is `docs/decisions/DEC-003-patch-lane.md`; this repo's
`decisions/DEC-003` is corpus storage. §10 asks for path disambiguation, and this
is the collision it warns about. The front-matter comment does disambiguate.

### What I could not settle

Whether the red-proof job has *ever* completed all five assertions in CI. It
failed in every run in which it existed prior to `96be26c`, and run
`32596678286` is the first in which it exited 0 — so the negative control,
assertions 3–5 and the severity run have one CI observation between them. They
pass locally under CI-parity clippy 0.1.98, which is the strongest available
evidence, but "green once" is where this gate's CI history currently stands.
