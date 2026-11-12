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
  id: HANDOFF-132
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-12
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-062

project:
  id: PROJ-004
  stage: STAGE-018
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-132: VERIFY: preview slice accessors on Container, zero-copy `&'a [u8]` (SPEC-062)

## Delegation Summary

You are the **cold verifier** for SPEC-062. You did **not** implement it. Review it
against SPEC-062 and DEC-057 and return exactly one verdict.

- **The build:** **PR #208**, branch `claude/spec-062-build`, cut from `origin/main`
  at `61f8e2b`. The head when this handoff was minted is `78e8382`. CI was 44/44
  green on that head.
- **Your branch:** your worktree is cut from `claude/spec-062-verify-base`. That is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under
  review is the build's code.

The build delivers:
- **`Container::previews()`** in `src/lib.rs`: an iterator of
  `Result<Preview<'a>, Error>`, one item per preview IFD, plus `Preview<'a>` and
  `PreviewKind`, both `#[non_exhaustive]`.
- **An allocation-free indexed tag reader and the preview walk** in `src/ifd.rs`
  (`uint_at` and friends). This is where the risk is: selection by
  `NewSubfileType`, the strip-table checks, and the byte-range arithmetic.
- **Two new `Error` variants** (`UnsupportedPreviewLayout`, `InvalidPreviewLayout`)
  classified in `Error::kind()`.
- **Tests:** `tests/previews.rs` (fixtures, both byte orders, the tier-B oracle, the
  staged-copy red-proof), `tests/previews_alloc.rs` (counting allocator), and rows
  in `tests/error_kind.rs`.
- **Corpus pins:** `[[file.preview]]` entries in `tests/corpus/manifest.toml`, 21
  previews across 11 DNGs, taken from exiftool 13.55. Loader changes are in
  `tests/support/corpus.rs` and `tests/corpus_manifest.rs`.
- **Fuzz:** the `ifd` target now drains `previews()`; two new seeds.
- **Docs:** CHANGELOG `[Unreleased]` and a provenance-ledger row.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-062-preview-slice-accessors-on-container-zero-copy-a-u8-dimensio.md`.
  Read all of it, especially the probe table, Design §1–8 and AC1–AC12.
- **Decisions:**
  - `./decisions/DEC-057-previews-are-newsubfiletype-1-or-65537-returned-as-one-contiguous-zero-copy-slice.md`
  - DEC-012: unknown is not no.
  - DEC-051: wildcard-free `Error::kind`.
  - DEC-052: public types are `#[non_exhaustive]`.
  - DEC-003: tier-B corpus bytes are never committed.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-131-build-preview-slice-accessors-on-container-zero-copy-a-u8-dimensio.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation is sound:
  - AC5's "length sum that overflows" is said to be unreachable as
    `ValueOverflow`; the fixture instead targets strips that look contiguous only in
    wrapping `u32` arithmetic and is refused as `NonContiguousStrips`;
  - the new fixtures are kept out of `tiff::all()` (to avoid editing
    `tests/ifd_reader.rs`) and written to `fuzz/seeds/ifd` via
    `tiff::preview_seeds()`;
  - the AC8 red-proof uses the staged-copy pattern from
    `tests/ljpeg_shape_binding.rs` and is not `#[ignore]`d;
  - `uint_at` refuses `RATIONAL` outright;
  - an unreadable optional tag is an `Err` item, while an absent one takes its
    default;
  - a `NewSubfileType` entry with count 0 reads as absent, so not a preview.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test`
  already passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use the
  files in place; never copy them into the repo.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC12 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
  - `msrv`
- Run `just decisions-audit --changed`.

### The behavioural surface (this is where defects escape)

**The pins are independent of irradiance.**
- Independently re-derive at least three previews, from three different files
  (include the K-3 III and one Compression 1 preview), outside the test code. Run
  `exiftool -a -G1 -s -n` and compare offset and length with the manifest. Then
  extract the bytes with exiftool (`-b -PreviewImage` or `-JpgFromRaw` as
  applicable) or with `dd`, MD5 them, and compare with an MD5 of the slice
  `previews()` returns, dumped by a few lines of standalone Rust in your scratchpad.
- Check the group mapping the pins rely on: `[IFD0]` is index 0, `[SubIFD]` 1,
  `[SubIFD1]` 2, `[SubIFD2]` 3. Confirm the walk order in code matches it.
- Confirm the tier-B test **ran** (no SKIP line for any DNG) and that the PEF is
  reported as not pinned, by name.

**Selection is exactly DEC-057.** Mutate it yourself, outside any test knob, and
confirm a test goes red for each mutation:
- accept bit 0 set (so 5 and 9 slip in);
- accept any non-zero value;
- drop 65537;
- treat an absent `NewSubfileType` as a preview;
- treat a count-0 entry as 1;
- select by size instead (for example, everything but the largest IFD).

**The byte range is right.** Mutate and confirm red:
- offset or length off by one;
- accept a gap between strips, or an overlap;
- accept out-of-order strips;
- take only the first strip of a contiguous multi-strip preview;
- compute the end in wrapping `u32`;
- skip the bounds check against `file.len()`.

**Iteration continues after an error.** A bad preview IFD is an `Err` item and the
next good preview is still yielded. Confirm a test pins this, and mutate the
iterator to stop at the first error.

**Hostile inputs are typed, never a panic.** For each Design §5 case, confirm the
typed error and its `ErrorKind`:
- tiled preview, `JPEGInterchangeFormat`, non-contiguous strips: Unsupported;
- mismatched strip-tag counts, zero strips: Corrupt (`InvalidPreviewLayout`);
- out of range: `Truncated`;
- an unreadable `NewSubfileType`: an `Err` item, not a silent skip.

Confirm no arithmetic can panic or wrap: offset plus length, strip count times
entry size, `u32` to `usize` conversions.

**The slice really borrows the input.** Confirm the lifetime is `'a` (the input),
not `&self`, and that the returned pointer lies inside the caller's buffer (the
build says it has a pointer check; read it).

**No allocation.**
- Read `tests/previews_alloc.rs`. Confirm the negative control proves the counter
  counts, that it drains both `Ok` and `Err` items, and that it covers
  `Container::previews()` itself rather than a helper.
- `git diff origin/main -- src/` must add no `Vec`, `Box`, `vec!`, `collect`,
  `to_vec`, `String` or `format!` on the preview path.

**No regression.**
- `git diff origin/main -- tests/corpus/manifest.toml` must be additions only.
- No existing test assertion changed, except `error_kind`'s required rows and
  counts (AC10). Check the 38 deleted lines in the diff and account for each.

**Error surface.**
- Both new variants are in `Error::kind()` with no wildcard.
- Each has `Display` naming its values, and a test of that text.
- `Preview` and `PreviewKind` are `#[non_exhaustive]` and their fields documented.
- CHANGELOG `[Unreleased]` names the additions.

**Fuzz.** Run the `ifd` target for at least 60 s with the new seeds (`just fuzz`
or `cargo +nightly fuzz run ifd`). Report the run count.

**CI is observed green on the PR head** with `gh pr checks 208`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are
  fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes
  (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.
- Open a NEW PR against `main` from `claude/spec-062-verify-<suffix>` carrying the
  handback, and any punch-list fixes. It must superset #208 and reference
  HANDOFF-132, SPEC-062, DEC-057 and PR #208.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
