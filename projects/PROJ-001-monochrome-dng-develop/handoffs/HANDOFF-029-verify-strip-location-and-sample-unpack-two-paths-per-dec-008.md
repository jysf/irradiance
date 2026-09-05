---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-029
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ✅ CONFIRMED, not corrected — this verify cycle ran
                                   #   on Opus 5. Read from my own transcript's
                                   #   `message.model`, which reports `claude-opus-5` on
                                   #   all 141 assistant records. FIRST time the dispatch
                                   #   hint has been right (build was 0 for 6); the hint
                                   #   is now 1 for 7, and this is the datapoint that
                                   #   makes it a rate rather than a streak.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-04
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-012

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 13821765           # REAL combined count — what cost-audit reads
  estimated_usd: 207               # ASSUMED Opus list rate x tokens_total, no cache discount
  duration_minutes: 18
  branch: feat/spec-012-strip-location-and-sample-unpack
  pr: null                         # NOT opened — return criterion 7 leaves it to the orchestrator
  completed_at: 2026-09-04         # YYYY-MM-DD
  notes: "Verdict ✅ APPROVED at 1606d4b (code-identical to branch tip 0f4cc38; that commit touches only the two handoff .md files). Real tokens_total deduped by message.id from this session's own transcript. ⚠ The spec's `cost.sessions` verify entry is deliberately NOT hand-written by this session, unlike HANDOFF-028's build: that hand-write is exactly what forced the orchestrator to hand-stamp HANDOFF-028's synced_at to avoid a fifth duplicate-entry occurrence. Leaving cost.sessions empty for verify means `just handback-sync SPEC-012` can be run ONCE, cleanly, from this block. handback-sync NOT run and PR NOT opened, per return criterion 7. Five follow-ups (FU-1..FU-5), no ship-blockers. tokens_total 13,821,765 = 91 distinct message.id records: 182 input + 56,338 output + 184,656 cache-creation + 13,580,589 cache-read; message.model reports claude-opus-5 on all 151 assistant records. estimated_usd is a DELIBERATE OVERESTIMATE per AGENTS.md §4 (tokens_total x list rate, no cache discount) AND the rate itself is ASSUMED, not confirmed: ~$15/MTok Opus-tier input list rate x 13.82M ~= $207. 98% of those tokens were cache reads, so a cache-aware figure lands closer to $25. Treat $207 as an order-of-magnitude ceiling."
  synced_at: 2026-09-04
---

# HANDOFF-029: Verify SPEC-012 — the unpack, at `1606d4b`

## Delegation Summary

Verify `SPEC-012` at **`1606d4b`** on `feat/spec-012-strip-location-and-sample-unpack`
(pushed, not merged; `main` at `a36582d`). **This is the first spec in the project
that produces pixels.**

⚠ **A worktree for this branch is live at
`~/PSeven/experiments/crustimg_redo_plus/irradiance-build-spec-012`.** Work there,
or in your own — do **not** try to check the branch out in the main checkout.

## ⚠ The headline result, measured by the orchestrator — and it changes your job

`SPEC-012` deliberately did **not** build the MD5 oracle (that is `SPEC-013`), so
the in-repo evidence for correctness is the first eight samples plus min/max.
The orchestrator built a throwaway probe against the shipped `unpack_into` and
compared the **whole plane** to `dnglab analyze --raw-checksum`:

| file | shape | whole-plane MD5 |
|---|---|---|
| `L1021223.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1026016.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1026192.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1000622.DNG` | 5216×3472, 16-bit | ✅ **match** |

Four for four, both `DEC-008` paths, two camera bodies, and every digest equals
the value already pinned in `tests/corpus/manifest.toml`. **The unpacker is
bit-exact today.**

So do not spend your round hunting for a wrong plane — it is right. Spend it on
**everything the checksum cannot see**: hostile input, the fuzz target's actual
reach, the error paths, and the claims that go beyond what was measured.

## What else the orchestrator reconciled

| claim | reconciled |
|---|---|
| branch + both commits on `origin`, CI green | ✅ `731a891`, `1606d4b` |
| first samples on both paths | ✅ `[746, 725, 711, 752, …]` and `[4761, 4591, 4622, 4363, …]` — identical to the design-time probe |
| `max <= WhiteLevel` holds | ✅ both files report `16383 <= 16383` — at the boundary, which is the interesting case |
| `DEC-016` shape | ✅ `unpack_into(&mut [u16])`, no allocation, length checked |
| fuzz seeds reach both paths | ✅ `valid-fourteen-bit.tiff` and `valid-sixteen-bit.tiff` both present |
| 110 tests, 0 failed | ✅ summed across targets |

## Where to look

1. **`AC7` — panic-freedom is the half a checksum cannot certify.** The plane is
   right on four good files; the spec's real risk is the other input space.
   Drive the fuzz target yourself and **say how you know it reached the 16-bit
   path**, not just that seeds exist. `SPIKE-001`'s blind spot was exactly a
   parameter that was always 14.
2. **`AC3` is the assertion that would have caught `SPIKE-002`.** Confirm it
   asserts the *measured impossible values* (43019, 39186) and not merely that
   two outputs differ. A test asserting "differs" passes for the wrong reason
   forever.
3. **`AC4` at the boundary.** Both real files hit `max == WhiteLevel` exactly.
   Does the check use `>` or `>=`? A `>=` would reject every honest Q2M frame.
   This is one character and the corpus sits right on it.
4. **`AC8`'s 182 MB.** The build measured peak RSS rather than estimating, which
   is what was asked. Sanity-check the method: plane is 94.9 MB and the input is
   86 MB, which sums suspiciously close to 182 — is the file being held entirely
   in memory alongside the plane, and is that a finding for `DEC-002`?
5. **The build reports catching a local/CI clippy-version gap.** Confirm what it
   was; that is `just lint-ci` doing the job `PATCH-001` created it for, and it
   is worth recording as evidence either way.

## One thing the orchestrator did to this branch

`HANDOFF-028`'s `synced_at` is **hand-stamped**, not written by `handback-sync`.
The build had already hand-written this cycle's cost session with the correct
figure, and running the script would have appended a **second identical entry** —
the bug `SPEC-003` first warned about, `SPEC-010/FU-2` hit with two identical
figures and `SPEC-009/FU-2` with a null beside a real one. **Fourth occurrence.**
Prevented rather than merged after the fact. The reason is inline in the field.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Observe CI green on the SHA you approve.**
2. **Watch a red-proof fail yourself** (§15 check 9).
3. **Fuzz (§12 bar 2 / §15 check 10) is the centre of this round**, not a
   formality — build claims 19.2 M combined runs across both paths.
4. **Provenance (§15 check 11):** confirm the ledger row exists with an honest
   class, and that `SPIKE-001`'s discarded decoder was **not** consulted — the
   handoff forbade it and `provenance-recorded-per-algorithm` is blocking.
5. Every mutation: **assert it changed the file and compiled** first. Stage your
   work before mutate-and-revert.
6. Handback with a real `tokens_total` **deduped by `message.id`** (read your own
   transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`), priced
   per-component at the rates for the model `message.model` reports.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

**Verdict: ✅ APPROVED at `1606d4b`** (`feat/spec-012-strip-location-and-sample-unpack`).
Branch tip `0f4cc38` is code-identical — it touches only `HANDOFF-028.md` and
`HANDOFF-029.md` — so the approval carries to the tip. Five follow-ups
(`FU-1`…`FU-5`), no ship-blockers.

### 1. Eleven gates + `just lint-ci`, run by me, in this worktree

```
 1. just build          cargo build --release                                    GREEN
 2. just test           corpus-status: 7/7 present — no tier-B test will skip
                        irradiance (lib unit)   52 passed
                        irr (bin unit)           0 passed (none defined)
                        corpus_manifest          9 passed
                        ifd_reader              12 passed
                        metadata_oracle         30 passed
                        plane_unpack             7 passed
                        doc-tests                0 passed
                        TOTAL 110 passed, 0 failed, 0 ignored, 0 SKIP lines     GREEN
 3. just lint           clippy --all-targets --all-features -D warnings + fmt    GREEN
 4. just typecheck      cargo check --all-targets --all-features                 GREEN
 5. just deny           cargo deny check licenses                                GREEN
 6. just deny-fuzz      cargo deny --manifest-path fuzz/Cargo.toml check ...     GREEN
 7. just lint-red-proof control clean (exit 0) → injection rejected (exit 101),
                        all five lints fired at the injected code, and still
                        fire without CI's -D warnings                            GREEN (red observed)
 8. just lint-no-allow  clippy --lib -F <five lints>                             GREEN
 9. just msrv           ~/.cargo/bin/cargo +1.90.0 check --all-targets           GREEN
10. just fuzz 60        ifd target, 13,572,814 runs / 61s, 0 crashes             GREEN
11. just fuzz-plane 120 plane target, 21,822,169 runs / 121s, 0 crashes          GREEN

    just lint-ci        clippy 0.1.98 (the floating CI toolchain) -D warnings    GREEN
```

`fuzz/artifacts/{ifd,plane}/` are both empty after the runs; `git status` is
clean (seed regeneration below included).

**CI observed green on the SHA I approve.** `gh run view 33933000222`
(`1606d4b`) — all nine jobs `success`: clippy, fmt, license policy, license
policy (fuzz graph), test, MSRV 1.90.0, lint policy red-proof, panic-free
policy, cost-capture audit. Also green on the tip `0f4cc38`
(run `33937275604`, same nine).

`just fuzz-seeds` regenerates all 26 `ifd` + 6 `plane` seeds byte-identically
to what is committed — the hand-mirrored `plane_fixture` has not drifted from
`tests/plane_unpack.rs`'s `Fixture`.

### 2. The plane, re-measured whole and independently

The build shipped only first-samples + min/max as in-repo evidence (`SPEC-013`
owns the oracle). I built a throwaway probe **outside** the worktree that calls
the shipped `plane::unpack_into` through the public API, dumps the whole plane
as native-LE `u16`, and `md5`s it:

| file | shape | ours | `dnglab --raw-checksum` | `manifest.toml` |
|---|---|---|---|---|
| `L1021223.DNG` | 8424×5632, 14-bit | `cb653b5b…` | `cb653b5b…` | ✅ same |
| `L1026016.DNG` | 8424×5632, 14-bit | `3f185125…` | `3f185125…` | ✅ same |
| `L1026192.DNG` | 8424×5632, 14-bit | `c7348179…` | `c7348179…` | ✅ same |
| `L1000622.DNG` | 5216×3472, 16-bit | `b0f602b9…` | `b0f602b9…` | ✅ same |

Four for four, both `DEC-008` paths, three-way agreement (us / the live tool /
the pinned manifest digest). **`max == 16383 == WhiteLevel` on all four** — the
corpus sits exactly on `AC4`'s boundary, which makes item 3 below the sharpest
check in this round.

### 3. Four mutations, each asserted to have changed the file and compiled first

Worktree was clean at `0f4cc38` before each, and reverted with
`git checkout -- src/plane.rs` after; `git status` clean between every one.

| # | mutation | result |
|---|---|---|
| 1 | `> white_level` → `>= white_level` (one character) | **RED, and the important one.** `unpacks_fourteen_bit_msb_first_samples` fails on `L1021223.DNG` sample 8397, `unpacks_sixteen_bit_in_file_byte_order` on `L1000622.DNG` sample 2178 — both `is 16383, which exceeds WhiteLevel 16383`. **The shipped code uses `>`; `>=` would reject every honest frame in the corpus.** The handoff's item 3 is answered by a measured red, not by reading the character. |
| 2 | `let byte_aligned = bits.checked_rem(8) == Some(0)` → `= false` (re-create `SPIKE-002`: read the 16-bit plane as a bit stream) | **RED.** `unpacks_sixteen_bit_in_file_byte_order` + `each_path_produces_impossible_values_on_the_others_data` + `a_plane_whose_max_exceeds_white_level_is_an_error` fail; the 14-bit test correctly stays green. `DEC-008`'s branch is load-bearing and the tests catch its removal. |
| 3 | `if let Some(white_level) = sensor.white_level` → `= None::<u32>` (delete `AC4`'s assertion) | **RED with a clean control:** exactly one test fails — `a_plane_whose_max_exceeds_white_level_is_an_error` — and the other **six stay green**. The red is attributable to the assertion, not to "something broke". `AC4`'s test has teeth. |
| 4 | `as_chunks::<1>/<2>` → `chunks_exact(1)/(2)` (reproduce the build's §2 claim) | **`just lint` GREEN (Homebrew clippy 0.1.97), `just lint-ci` RED (clippy 0.1.98):** `error: using chunks_exact with a constant chunk size … #chunks_exact_to_as_chunks`, twice, `src/plane.rs:185` and `:192`. The build's account is exact, and `lint-ci` is doing the job `PATCH-001` created it for on live code rather than as an anecdote. |

### 4. Fuzz — the centre of the round, and the inference replaced with a measurement

`just fuzz-plane 120` → **21,822,169 runs, 0 crashes, 0 artifacts** (seed corpus
405 files after prior runs; `cov: 599 ft: 1530` at exit).

The build said it knew both paths were reached because "the seed corpus plants
one fixture per path … `cov: 597` is stable across both runs, *consistent with*
both being live inputs." That is an inference. I replaced it with region
coverage of the actual instrumented fuzz binary over the actual corpus
(`cargo +nightly fuzz coverage plane`, merged with Homebrew's `llvm-profdata`
— the nightly toolchain has no `llvm-tools` component installed, which is why
`cargo fuzz coverage` errors out on this machine and is worth knowing):

```
src/plane.rs   Regions 251, 76.89% covered   Functions 11, 72.73% executed

  156|      6| fn unpack_bitstream(...)          ← sub-byte path:      6 executions
  172|     16| fn unpack_byte_aligned(...)       ← byte-aligned path: 16 executions
  192|       |     16 => {
  193|     16|         let (chunks, _remainder) = strip.as_chunks::<2>();
  194|    105|         for (chunk, out) in ...    ← 105 samples converted
  311|     22|     if byte_aligned {
  312|     16|         unpack_byte_aligned(...)   ← 16
  314|      6|         unpack_bitstream(...)      ←  6
```

**Both of `DEC-008`'s paths are reached by the fuzz corpus, measured.**
`SPIKE-001`'s blind spot is not recreated at the `bits % 8` level. The same
profile is what surfaced `FU-1` and `FU-2` below — the two places it *is*
recreated one level down.

`just fuzz 60` (the `ifd` target, re-run because `SPEC-012` added five `Error`
variants): 13,572,814 runs, 0 crashes.

### 5. `AC8` reproduced, and the answer to the handoff's question about it

`/usr/bin/time -l ./target/release/irr unpack`, this machine, this build:

| file | input file | plane | measured peak RSS |
|---|---|---|---|
| `L1021223.DNG` (47 MP, 14-bit) | 85,796,864 | 94,887,936 | **182,435,840** |
| `L1000622.DNG` (18 MP, 16-bit) | 36,433,408 | 36,219,904 | 74,399,744 |

Both numbers reproduce the build's to the byte. The accounting is sound:
85,796,864 + 94,887,936 = 180,684,800, leaving 1,750,000-odd bytes for the
binary, the `Sensor`'s `Vec`s and allocator overhead. **Yes, the input file is
held whole alongside the plane** — and the method was measurement, as asked, not
estimation. `unpack_into` itself allocates nothing; I confirmed it by reading
every path. What is *not* recorded anywhere is the API-shape consequence, which
is `FU-4`.

### 6. Findings

Every one is a follow-up. Nothing here lets bad data or a panic reach a
consumer, and I looked specifically for that.

| id | finding | why not ship-blocking | suggested disposition |
|---|---|---|---|
| `FU-1` | **`bits = 8` and `bits = 12` are in `SUPPORTED_BITS`, reachable from untrusted input, and executed by nothing.** Coverage: `unpack_byte_aligned`'s `8 =>` arm is **0 executions** across 21.8 M fuzz runs, and no test drives `unpack_into` at 8 or 12 either (fixtures use 16, 14, 10; the only 8-bit test drives `BitReader` directly, not the path). This is `SPIKE-001`'s "the parameter was always 14" one level down: four widths declared, two exercised. | I drove both through the real API rather than only reporting the hole: 8-bit → `[1,2,3,4]`; 12-bit `AB CD EF` → `[2748, 3567]` = `0xABC`/`0xDEF`, hand-derived; byte order correctly irrelevant to both; and an 8-bit plane over a `WhiteLevel` errors loudly. **Both are correct.** It is test debt, not a defect. | `fixed` in a punch-list round — two tier-A cases in `tests/plane_unpack.rs` + two seeds in `plane_seeds()`. Cheap, and it closes the spec's own headline failure mode at the width level. |
| `FU-2` | **The fuzz target never once exercises `Error::SampleExceedsWhiteLevel`** — the single assertion `DEC-008` calls the one that caught `SPIKE-002`. Coverage: the `if let Some(white_level)` body is **0 executions**. Cause: `examples/fuzz-seeds.rs::plane_fixture` has no `white_level` parameter at all, unlike `tests/plane_unpack.rs`'s `Fixture`, so no seed carries tag 50717 and libFuzzer would have to invent a whole valid IFD entry to reach it. | The assertion has tier-A coverage (`AC4`) and I proved that test has teeth (mutation 3). The uncovered code is one comparison and one return — no panic surface. | `fixed` — give `plane_fixture` the `white_level` field its test-side twin already has and add one seed. Same edit as `FU-1`; the build's own reflection already proposes the shared `tests/support/plane_fixture.rs` that would prevent both. |
| `FU-3` | **`DEC-002` is still `status: proposed` and carries no note that a spec now depends on it.** `just decisions-audit` says so in as many words — *"accept the DEC, or note in it that a spec already depends on it"* — and neither was done. `DEC-016` exists *specifically* because `DEC-002` is unresolved, and `DEC-002` does not mention `SPEC-012` or `DEC-016`. | Advisory in the audit; confidence 0.72 is above §16's 0.6 yellow-flag line. | `fixed` at ship — one back-reference in `DEC-002`, or accept it. |
| `FU-4` | **`AC8`'s peak RSS is 47% input file, and that half is the API contract, not `irr`'s convenience.** `unpack_into` takes the whole file as `&[u8]` and indexes it at *absolute* strip offsets, so every caller must make the entire file addressable — 85.8 MB here — on top of the 94.9 MB plane. `DEC-016` settles the *destination* allocation and calls `AC8`'s number "mostly the caller's buffer"; neither it nor `DEC-002` records that the *source* side forces whole-file addressability, with `mmap` as the only escape and it undocumented. | Addressability, not residency — an `mmap`ing caller pays pages, not RSS. Nothing is wrong; it is unrecorded. | `spec:` the `DEC-002` resolution, or a `Consequences` line in `DEC-016`. This is my answer to the handoff's "is that a finding for `DEC-002`?" — yes, as a follow-up. |
| `FU-5` | **`Error::Truncated`'s `at` field carries two different coordinate systems.** `src/plane.rs:308` reports a file offset; `src/plane.rs:112` (inside `BitReader`) reports a *strip*-relative byte position under the same variant. A consumer diagnosing a truncated file cannot tell which it got. | The `BitReader` arm is in fact unreachable once layer-0 has passed (the strip holds exactly `w × h × bits` bits, so `read` never over-runs) — it is defensive only. | `fixed` — either make it file-relative or give it its own variant. Low value, but it is a public error surface and one `file:line`. |

`docs/provenance-ledger.md`'s `src/plane.rs` row says "fuzzed 13,050,886 runs
(45 s)" — the first of the build's two runs, not the 19,184,408 combined it
reports elsewhere. Not filed as a finding; worth correcting to the honest
number at ship, when the ledger is confirmed current anyway.

### 7. The four repo-specific verify checks (§15 9–12)

9. **Red-proof observed personally.** `just lint-red-proof`: control clean
   (exit 0) → injection rejected (exit 101) → all five lints fired at the
   injected lines and still fire without `-D warnings`. Plus the three source
   mutations in §3, of which #3 is a red with a six-test negative control.
10. **Fuzz exists and ran** — §4. 21.8 M runs on the new target, both paths
    reached with coverage as evidence, plus 13.6 M on `ifd`.
11. **Provenance row present and honest.** `docs/provenance-ledger.md` gains a
    `src/plane.rs` row: source TIFF 6.0 (1992) §Compression=1 + `DEC-008`,
    licence "public specification", **class 1 — specification**. That class is
    right: `DEC-008` itself carries the byte evidence and derives the packing
    rule from the spec, not from an implementation. `SPIKE-001`'s decoder was
    **not** consulted, and this is structurally checkable rather than a promise —
    the spike was dispositioned `discarded`, its code is not in the tree, and
    `spikes/done/SPIKE-001-*.md` contains no unpacker source (its only fenced
    block is `OpcodeList3` coefficients). `src/plane.rs`'s module header states
    the same provenance at the point of use.
12. **No new dependency.** `Cargo.toml`/`Cargo.lock` are untouched by this
    branch; `fuzz/Cargo.toml` gains only a `[[bin]]` entry. Both `cargo deny`
    invocations green.

Also checked: no drift from `DEC-008` (branch is `bits % 8`, byte order applied
to the aligned path only — and `AC3`'s case 2 actually *executes* the `MM`
aligned path, which `DEC-008` recorded as reasoned-about-but-never-run),
`DEC-012`, `DEC-016`; `just decisions-audit` reports **0 structural errors**
(the four scope warnings are `DEC-000`/`DEC-012`+`DEC-015`/`DEC-013`+`DEC-014`,
all pre-existing); `just cost-audit` green; all seven `## Failing Tests` names
resolve to **exactly one** test each across all targets (`--exact --list`), so
none is the vacuous zero-match the spec warns about; the build's reflection is
answered substantively, not mailed in; `cost.sessions` carries design
(null-with-note, legitimate for main-loop) and build (29,580,529, real).

`Sensor::samples_per_pixel` being ignored by `unpack_into` is **not** a finding:
`Container::is_sensor_ifd` only ever returns a match when `SamplesPerPixel == 1`,
so a multi-sample `Sensor` cannot be constructed. Checked rather than assumed.

### 8. Where the round went

The orchestrator's instruction to stop hunting for a wrong plane was right, and
I confirmed it independently anyway (§2) because it was cheap and it is the
strongest single fact about this spec. The round's value was in the two things
a checksum cannot see: the `>` / `>=` boundary, which the corpus sits exactly
on and which one mutation settles (§3.1), and the fuzz target's actual reach,
where coverage turned a plausible inference into two measured numbers — and, in
the same profile, found the two widths and the one assertion that nothing
executes (`FU-1`, `FU-2`).
