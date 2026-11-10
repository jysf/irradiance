---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-052
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: S
  verify_verdict: approved

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-045, DEC-046]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-039]

depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

claimed_by: null

value_link: "makes the container parser safe to point at hostile metadata — the pre-0.2.0 'safe to depend on' thesis"

cost:
  tokens_estimate: 20000000
  sessions:
    - cycle: build
      handoff: HANDOFF-109
      agent: claude-opus-5
      interface: other
      tokens_total: 7025673
      estimated_usd: 5.29
      duration_minutes: 21
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns."
    - cycle: verify
      handoff: HANDOFF-113
      agent: claude-opus-5-5
      interface: other
      tokens_total: 4376521
      estimated_usd: 3.15
      duration_minutes: null
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 verify. Verdict APPROVED at 75f681a, no code change needed. A counting-allocator probe confirmed the guard cuts a 790 KB hostile file from 402 MB to 787 KB; both corpus DNGs still parse; red-proof reproduced. tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  totals:
    tokens_total: 11402194
    estimated_usd: 8.44
    session_count: 2
---

# SPEC-052: Parser resource guard — aggregate entry/byte budget (group A)

## Context

Pre-0.2.0 code-review finding **#1 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted into the 0.2.0
wave by DEC-045. `walk_chain` (`src/ifd.rs:905`) de-duplicates only *identical*
IFD offsets (`visited.contains`, `src/ifd.rs:922`) and caps IFD *count*
(`MAX_IFDS = 512`, `src/ifd.rs:69`), but nothing caps the *aggregate* number of
entries or bytes parsed. `read_ifd` (`src/ifd.rs:955`) does
`Vec::with_capacity(count)` of up to 65,535 entries per IFD (`count` is a `u16`),
independently for each distinct offset, and bounds each table only to lie *inside*
the file — so entry tables at **distinct** offsets may **overlap** the same bytes.
A chain of distinct offsets therefore reaches a ~33.5M-entry ceiling
(512 × 65,535); the reviewer measured ~6.3M entries (~72 MiB) from a ~770 KB file.
`Entry` is a flat 12-byte struct with no heap payload (`src/ifd.rs:339`), so the
walk's memory is `total_entries × ~12 bytes` — a metadata-amplification DoS at the
container boundary, catchable by nothing the pipeline has.

This is the IFD-walk analogue of the plane-level validate-before-allocate work
(SPEC-039 / group E), which is why group E's unified preflight depends on this
landing first.

## Design

**The decision is DEC-046.** During the walk, accumulate a running total of each
IFD's entry-table size (`count × ENTRY_SIZE`, `ENTRY_SIZE = 12`); reject the file
the moment that running total would exceed the file length. That caps total parsed
entries at `file_len / 12` and closes the amplification, with **no magic
threshold** and **no change to any real file** — measured, the corpus DNGs' tables
are ~0.003 % of file length (DEC-046 "Measured baseline"). It is the same idiom as
the plane-level aliasing rule SPEC-039 proposes.

### 1. Where the check lives

Inside `read_ifd` (`src/ifd.rs:955`), **after** `table_len` is computed
(`src/ifd.rs:960`) and **before** `Vec::with_capacity(count)` (`src/ifd.rs:978`),
so an over-budget IFD allocates nothing. Thread a `&mut u64` running total from
`Container::parse` (initialise to `0`, `src/ifd.rs:872` area) through `walk_chain`
(`src/ifd.rs:905`) into `read_ifd`. The exact carrier — an extra `&mut u64`
parameter, or a small walk-state struct that also carries `visited` — is the
implementer's choice, provided it adds **no allocation** and keeps `read_ifd`
reading `&self.data`.

### 2. The rule

```text
running_total = running_total.checked_add(table_len as u64)   // table_len = count × 12
if running_total > data.len() as u64:
    return Err(Error::IfdTableBytesExceedFile { declared: running_total, file_len: data.len() })
```

`checked_add` guards the running total itself (the five deny lints forbid a bare
`+`). `declared` is the running total at the point of rejection; `file_len` is
`data.len()`. The comparison is strict `>` — a file whose IFD tables sum to exactly
the file length (a metadata-only TIFF) is still accepted.

### 3. The new error variant

`Error::IfdTableBytesExceedFile { declared: u64, file_len: usize }` in `src/lib.rs`
(additive; `Error` is `#[non_exhaustive]`, `src/lib.rs:217`). Add a `Display` arm
modelled on `TooManyIfds` (`src/lib.rs:763`), e.g.
`"IFD entry tables declare {declared} bytes, exceeding the {file_len}-byte file"`.
It is a **Corrupt**-class error (a hostile/malformed file, not caller misuse) — a
note for SPEC-040's future `Error::kind` table, no classification code here.

### 4. What stays unchanged

`MAX_IFDS`, `MAX_IFD_DEPTH`, `MAX_TAG_VALUES`, the `visited` cycle guard, and how
value payloads are read later are all untouched. This is a **third, orthogonal**
guard. The regression (§Failing Tests) uses **distinct** offsets so it exercises
*this* guard, not the cycle guard (triage requirement).

## Inputs

- **`src/ifd.rs`** — `Container::parse` (l.848), `walk_chain` (l.905), `read_ifd`
  (l.955, the `table_len`/`Vec::with_capacity` site), `MAX_IFDS` (l.69),
  `ENTRY_SIZE` (l.81), `Entry` (l.339, flat 12 bytes).
- **`src/lib.rs`** — `Error` (`#[non_exhaustive]`, l.217); the IFD-limit variants
  (`TooManyIfds` l.258, its `Display` l.763) to model the new variant.
- **`tests/support/tiff.rs`** — `Ifd::new(offset, entries, next)` (l.106),
  `tiff(order, ifd0_offset, ifds)` (l.120, grows to fit chosen offsets),
  `at_offset(tag, field_type, count, offset)` (l.88), `short`/`long`. A small
  fixture helper for an aliased-table IFD chain may be added here.
- **`fuzz/fuzz_targets/ifd.rs`** — the target to extend; `examples/fuzz-seeds.rs`
  for a new aliased-table seed.
- **Corpus** — `IRRADIANCE_CORPUS_DIR` (see memory); the two DNGs must still parse.

## Outputs

- **Files modified:** `src/ifd.rs` (the running-total guard in `read_ifd`, threaded
  from `parse`/`walk_chain`); `src/lib.rs` (one new `Error` variant + `Display`
  arm); `fuzz/fuzz_targets/ifd.rs` (the guard runs on every input);
  `examples/fuzz-seeds.rs` (an aliased-table seed); `CHANGELOG.md`
  (`[Unreleased] → Added`/`Security`); `docs/provenance-ledger.md` (one row: own
  prior code, class 3 — a guard added to `walk_chain`/`read_ifd`).
- **Files created:** `tests/ifd_resource_guard.rs` — the amplification, bound,
  corpus and red-proof tests.
- **New public API:** `Error::IfdTableBytesExceedFile { declared: u64, file_len: usize }`.
  No other signature changes (the running-total carrier is private).

## Acceptance Criteria

- [ ] **Amplification rejected:** a fixture with many IFDs at **distinct** offsets
      whose entry tables overlap the same bytes, so the sum of declared table bytes
      exceeds the file length, returns `Err(IfdTableBytesExceedFile { .. })` — and
      the parse allocates no more than `~file_len / 12` entries total (the test does
      not depend on the 33.5M ceiling being reachable).
- [ ] **Bound holds:** whenever `Container::parse` returns `Ok`, the sum of all
      walked IFDs' `count × 12` is `<= data.len()`.
- [ ] **Distinct from the cycle guard:** the amplification fixture uses distinct
      offsets, so removing the `visited` cycle check would NOT catch it — only the
      new aggregate guard does (asserted by the red-proof mutating the aggregate
      guard, not the cycle guard).
- [ ] **No false positive:** every existing tier-A fixture (valid single/multi-IFD,
      SubIFD, CFA) parses unchanged, and both tier-B corpus DNGs parse (skip-loud
      when the corpus is absent, naming the missing file).
- [ ] **Panic-free:** no `unwrap`/indexing/`as`-overflow/unchecked arithmetic; the
      running total uses `checked_add`; five deny lints + `lint-no-allow` clean; the
      fuzz target runs the guard on every input and runs clean for 60 s
      (`just fuzz`).
- [ ] **Red-proof:** the amplification test goes **red** against a mutated copy of
      `src/ifd.rs` with the running-total guard removed (returns `Ok` instead of the
      typed error), with the unmutated control run first (`oracle-must-be-shown-red`,
      DEC-009).

## Failing Tests

- **`tests/ifd_resource_guard.rs`**
  - `"aliased_ifd_tables_exceeding_file_are_rejected"` — asserts: a chain of N
    distinct IFD offsets whose entry tables overlap one region (sum of declared
    `count × 12` > `data.len()`) → `Err(IfdTableBytesExceedFile { .. })`. Build it
    from `tiff()` + `Ifd::new` with offsets a few bytes apart (a small
    `aliased_ifd_chain` helper in `tests/support/tiff.rs` is acceptable).
  - `"aggregate_table_bytes_within_file_still_parse"` — asserts: several IFDs whose
    tables are disjoint (or overlap but still sum to `<= data.len()`) parse `Ok`,
    pinning that the rule is the **sum vs file length**, not overlap-detection.
  - `"ok_implies_table_bytes_bounded"` — asserts: for each passing fixture, the sum
    of `ifd.entries().len() × 12` over `container.ifds()` is `<= data.len()`.
  - `"distinct_offsets_isolate_the_new_guard"` — asserts: the amplification fixture's
    offsets are all distinct (no two equal), so the `visited` cycle guard cannot be
    what rejects it.
  - `"corpus_dngs_not_rejected"` — asserts: both tier-B DNGs parse `Ok`; skips loud,
    naming the file, when `IRRADIANCE_CORPUS_DIR` is unset/absent.
  - `"ifd_guard_red_proof"` — asserts: replaying a committed copy of `src/ifd.rs`
    with the running-total guard deleted makes `"aliased_ifd_tables_exceeding_file…"`
    fail (parse returns `Ok`); control (unmutated) run first per DEC-009. Reuse the
    `stage_probe_crate` mutated-copy apparatus (copy it rather than refactoring).
- **`fuzz/fuzz_targets/ifd.rs`** — extended so the guard runs on every input; assert
  the §Acceptance "bound holds" property whenever `parse` is `Ok` (sum of table
  bytes `<= data.len()`). Add an aliased-table seed in `examples/fuzz-seeds.rs`.

## Non-Goals

- No change to `MAX_IFDS`, `MAX_IFD_DEPTH`, `MAX_TAG_VALUES`, the cycle guard, or
  value-payload reading.
- Not the *pixel* allocation budget — that is `plane::validate` (SPEC-039). This
  guard is the metadata-parse allocation only.
- No absolute `MAX_TOTAL_IFD_ENTRIES` cap (DEC-046 rejected it as a redundant magic
  number).
- No new container features; no `Error::kind` classification code (SPEC-040).

## Notes for the Implementer

- Read `## Design` and **DEC-046** first. The guard is ~10 lines; the care is in the
  threading, the `checked_add`, and the fixture that isolates the new guard from the
  cycle guard (distinct offsets).
- `just test` (not `--all-features`); five deny lints are on.
- Fuzz: `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd …`
  or `just fuzz` (see `fuzz_targets/ifd.rs` header for the `+toolchain` trap).
- Run the tier-B corpus locally (`IRRADIANCE_CORPUS_DIR`, see memory) to confirm the
  guard rejects nothing real; record the result in the handback.
- `irr ifd <file> --entries` dumps a file's IFDs if you want to eyeball a baseline.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary:
   `design` | `build` | `verify` | `ship` | `escaped` | `none`.
   — <one word>

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after. Write `none` if this spec has no user-visible outcome.
   — <answer | none>
