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
  id: HANDOFF-022
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT from tier_map.verify, NOT a record.
                                   #   tier_map is now 0 FOR 3 (SPEC-007/FU-6): SPEC-005's
                                   #   build ran on claude-sonnet-5 against a map saying
                                   #   opus. CORRECT THIS to what actually ran, and check
                                   #   your own message.model rather than assuming.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-22
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-005

project:
  id: PROJ-001
  stage: STAGE-001
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-022: Verify the metadata oracle — SPEC-005 at `418be15`

## Delegation Summary

Verify `SPEC-005` at **`418be15`** on branch **`feat/spec-005-metadata-oracle`**
(not merged; `main` is at `04aaf4b`). Nine acceptance criteria, nine named tests,
87 tests total across six targets, `src/` untouched.

**This handoff carries FOUR findings the orchestrator surfaced during
reconciliation, and one process irregularity. They are not conclusions — they are
required checks. Confirm or kill each one yourself.** Where I state a
measurement, reproduce it; where I state a reading of the code, read it. If you
think I am wrong, say so with the command that shows it — that has happened
before here and it was the right outcome both times.

## ⚠ Process irregularity, disclosed up front

The build session **reported done but did not finish the cycle**: it left
`HANDOFF-021`'s `handback:` block entirely null, did not branch, did not commit,
and asked the orchestrator to run `/cost` — a client-side command the assistant
cannot execute, and which would have measured the *orchestrator's* session, not
the build's.

The orchestrator therefore finished the **mechanical remainder** per `DEC-004`
rule 1: branched, committed, recovered the real token figure from the build
session's own transcript, and filled the handback. **The code is entirely the
build's; the commit and the handback are not.** Weigh that when you judge
`§15` check 6 (implementer reflection) — there is no build reflection to read,
which is itself worth a finding.

## Context the Receiving Agent Needs

Read `SPEC-005` in full (its `## Implementation Context` is a measured probe, not
background), `AGENTS.md` §12 and §15, `guidance/constraints.yaml`,
`decisions/DEC-012` and the **new** `decisions/DEC-013` — note that
`docs/decisions/DEC-013` is a *different file in the template's namespace*
(§10). Corpus: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.

## What the orchestrator already re-ran (reproduce, do not assume)

- `cargo fmt --check`, `clippy --all-targets --all-features -D warnings`,
  `just msrv`, `just deny`, `just deny-fuzz`, `just lint-red-proof`,
  `just lint-no-allow`, `just cost-audit`, `just validate` — **all exit 0**.
- `cargo test --all-features` → **87 passed, summed across all six targets**
  (`45+0+9+12+21+0`), corpus present.
- `git diff --stat` on `src/`, `Cargo.toml`, `Cargo.lock` — **empty**.
- Both red-proofs **read line by line** and judged genuine: tier A perturbs
  `bits_per_sample`→13 and asserts *exactly one* mismatch named `BitsPerSample`
  against an empty-diff control; tier B XORs `ActiveArea`'s payload in an
  in-memory copy, **asserts the patch changed the buffer**, asserts one mismatch
  named `ActiveArea`, then re-diffs unpatched as its control. Nothing is written
  to disk.

## THE FOUR FINDINGS — confirm or kill each

### F-1 — `diff()`'s `malformed_tags` guard is dead code

Measured by the orchestrator: comment out the `!sensor.malformed_tags.contains(...)`
condition at `tests/support/tools.rs:350` and **all 21 oracle tests stay green**
(mutation asserted applied by `diff`, tree restored byte-identical). Reproduce it.
If nothing dies when a guard is removed, the guard is not guarding.

### F-2 — `DEC-013`'s premise appears to be false

`DEC-013` says `K3III.DNG` "would fail `AC1`'s own test on `BlackLevelRepeatDim`
forever". Check whether it would. `exiftool` reports a bare `1` for that tag;
`tools.rs:247-248` runs `<[u32;2]>::try_from(v.as_slice()).ok()`, which on a
one-element vector yields **`None`**; our reader also reports `None`
(`DEC-012` drops the value). `None == None` — no mismatch, so the permanent red
the decision exists to prevent may never have been possible. **This is the
finding I am least certain of** — a decision record is a serious artifact and I
would rather be corrected than have it quietly stand. Verify it directly.

### F-3 — `DEC-013` records choosing Option C and appears to ship Option B

Option C is quoted in the record as *"`diff()` reads `malformed_tags` and skips
exempted fields **generically** … no per-file knowledge in the comparator at all
… needs no update when a FUTURE file exercises a different malformed tag."*
`tools.rs:350` reads `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM)`
— one hardcoded tag. `grep -n 'malformed_tags' tests/support/tools.rs` returns
exactly one code site. If that reading holds, this is the same shape as
`SPEC-008/FU-4` and would be the **fourth** instance of
`measurement-over-generalised`, which is already at its bar.

### F-4 — the oracle may not distinguish "absent" from "unparseable"

`ActiveArea`, `DefaultCropOrigin` and `DefaultCropSize` all parse via
`values_for(...).and_then(|v| match v.as_slice() { [..] => Some(..), _ => None })`.
A **garbled** tool reading and an **absent** tag both become `None`, so a garbled
reading silently *agrees* with a `None` on our side. `AC2` exists for exactly
this: *"An oracle that ignores `None` cannot catch a reader that invents values."*
Judge whether `AC2` is met as written. **The spec did not anticipate this and
neither did I** — if this is real it is a design gap as much as a build one, and
should be labelled that way.

### F-5 — three tier-B tests compute a coverage counter and never assert it

`tests/metadata_oracle.rs:94`, `:137`, `:194` each maintain `checked` and report
it via `eprintln!`, which `cargo test` swallows without `--nocapture`
(measured in `SPEC-002/F2`). Measured: with `IRRADIANCE_CORPUS_DIR` pointing at a
nonexistent path, `metadata_matches_exiftool_on_every_corpus_file` **passes
having checked zero files**. `AC1` says "on all seven files". This is
`named-tests-can-pass-vacuously` (an `accepted` signal) occurring inside the
spec whose whole job is to stop things passing vacuously.

## Your own checks — do NOT limit yourself to my list

The most valuable thing you can do is find a **sixth**. Two suggestions, both in
this repo's grain:

1. **Is `diff()` narrower than it looks?** It has eleven explicit comparisons.
   Perturb each of the eleven fields in turn on the tier-A fixture and confirm
   each one produces a mismatch. A field that is compared but whose *reading*
   side is never populated compares `None` to `None` forever. This is
   `SPEC-008/FU-1`'s shape one level up.
2. **Is the dnglab uniqueness assertion real?** The build claims each key's match
   count is asserted unique before use. Plant a duplicate key and watch it
   refuse, per `attribute-text-inside-doc-comments`' general form.

## Return Criteria

1. Ten gates re-run **by you** and pasted. Sum test counts **across all six
   targets** — a zero-match `cargo test <name>` exits 0.
2. **Watch both red-proofs fail yourself** (§15 check 9). A red you did not
   personally observe failing is a self-report.
3. **Fuzz (§15 check 10)** — the build claims 13,455,965 execs, seeds unchanged.
   Re-run it: `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd`.
4. Confirm each of the nine named tests **exists** per-target via `-- --list`
   before trusting any green.
5. Every mutation you run: **assert it changed the file, and assert it compiled,**
   before drawing a conclusion. Five failures here, historically.
6. Fill the `handback:` block including a **real `tokens_total`, deduped by
   `message.id`**, and say you deduped. You *can* get this — read your own
   transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`; the session id is
   in the scratchpad path in your system prompt. Compute `estimated_usd`
   **per-component at the rates for the model that actually ran** — check
   `message.model`, do not trust `tier_map`. On this spec's build, Opus rates
   would have overstated by **5.0×** and the repo's flat `rate_per_mtok` by
   **14.7×**.
7. **Correct `handoff.to_agent`** to what actually ran.
8. Do **not** run `just handback-sync` — the orchestrator runs it.
9. Label findings `SB-N` / `FU-N` for **this** spec from 1. Each will be
   dispositioned at ship into `fixed` / a spec / a signal / an explicit close
   (§15, *Where an unresolved follow-up goes*) — say which of the four you think
   each wants.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

*(Filled by the reviewer. Mirror the `handback:` front-matter above.)*
