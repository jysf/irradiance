---
# Maps to ContextCore handoff.* semantic conventions.
# BUILD (Phase 1) of SPEC-042, the 0.2.0 release. Rationale + handback contract:
# docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-123
  cycle: build
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-042

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: null
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null
  notes: null
  synced_at: null
---

# HANDOFF-123: Build the 0.2.0 release prep (SPEC-042, group F, Phase 1)

## Delegation Summary

Phase 1 of the 0.2.0 release: the **reversible prep PR** — seal the last public
parser-output types, bump the version, promote the CHANGELOG, write the release notes,
and attach the real-file corpus evidence. **You do NOT tag or publish** — that is Phase
2, human-gated. The whole decision is **DEC-052**; read it and SPEC-042's `## Build work`
first. Do not cut a partial release and do not claim colour accuracy.

## What to read first

- **Decision:** `./decisions/DEC-052-seal-the-remaining-public-parser-output-types-and-cut-0-2-0-as-semver.md` — the seal, semver/0.2.0, the deferred follow-ups (do NOT implement the two deferred opcode items here — they are SPEC-056), the no-colour-accuracy call.
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md` — `## Release Scope`, `## Build work`, the `## Release Pre-Flight` checklist (fill the instance commands + `[now]`/`[cut]`/`N/A` timing).
- **Decisions:** `DEC-028` (publish checks, metadata), `DEC-006` (CHANGELOG → version), `DEC-003` (tier-B never committed), `DEC-045`/`DEC-049` (the seal began with `Sensor`).
- **Code:** `src/ifd.rs` (`Cfa` `:616`), `src/opcode.rs` (`Opcode` `:105`, `WarpRect` `:155`), `Cargo.toml` (the `test-fixtures` feature + `[dev-dependencies]` + `[package]` version/metadata), `.repo-context.yaml` (`spec.version.scheme`), `tests/` (the `Cfa`/`WarpRect`/`Opcode` literal fixtures), `CHANGELOG.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md`; `docs/versioning.md`.

## Expected Deliverables (Phase 1)

- `#[non_exhaustive]` on `Cfa`, `WarpRect`, `Opcode`; in-crate matches still compile (no crate-wide wildcard added); the `tests/` fixtures for all three migrated onto the dev-only `test-fixtures` feature (extend it with constructors).
- A self dev-dependency (`irradiance = { path = ".", features = ["test-fixtures", "irr"] }`) so a bare `cargo test` compiles the integration tests.
- `Cargo.toml` `version = "0.2.0"` + publish-ready metadata (DEC-028); `.repo-context.yaml` `spec.version.scheme: semver`.
- `CHANGELOG.md`: `[Unreleased]` promoted to `## [0.2.0] - <date>` with a fresh empty `[Unreleased]`.
- Release notes (user-facing headline + **Known limitations** naming the two deferred opcode follow-ups and the absence of a colour-accuracy claim).
- The `## Release Pre-Flight` checklist filled with instance commands and honest `[now]`/`[cut]`/`N/A` timing; `cargo publish --dry-run` clean (`[now]`).
- **Tier-B corpus evidence run on the candidate SHA** recorded in the handback (`IRRADIANCE_CORPUS_DIR`, every held file bit-exact); `just package-targets-check`, `just deny`, `just deny-fuzz`, `just test` (`--all-features`), `just lint` all green.
- A PR against `main` from `claude/spec-042-build-<suffix>` referencing HANDOFF-123, SPEC-042, STAGE-010, PROJ-002, DEC-052 / DEC-028. The PR description says **"Phase 1 — prep-complete, cut-deferred"** and lists the Phase-2 steps for the human cutter.

### If `cycle: verify` — the verify contract

Review cold. Confirm the three types are sealed and in-crate matches compile; the fixtures
moved onto `test-fixtures` and a bare `cargo test` builds; `version = 0.2.0` + semver
scheme; CHANGELOG promoted; release notes name the known limitations and make no
colour-accuracy claim; `cargo publish --dry-run` is clean; the corpus evidence is attached
and bit-exact; no tag/publish happened (Phase 2 is deferred). `just decisions-audit
--changed`. Output exactly one of ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- **Phase 2** — the `v0.2.0` tag and `cargo publish` (human-gated). Do not create a tag or publish.
- The two deferred group-C opcode follow-ups (SPEC-056, 0.2.1).
- Any colour-accuracy claim (SPEC-035 is hardware-gated); any new capability; STAGE-008 work.

## Return Criteria

Fill the `handback:` front-matter (a real `tokens_total`) and the `## Handback` section
(with the corpus-evidence result and the filled pre-flight timing); set
`handoff.status`/`handback.status` → `completed`; open the Phase-1 PR. Report token usage
even if blocked.

## Handback

### Execution notes
- **Branch / PR:** [link]  **Completed at:** YYYY-MM-DD  **All Phase-1 deliverables met?** yes/no
- **Corpus evidence (candidate SHA):** every held tier-B file bit-exact? (the result)
- **Pre-flight:** which items are `[now]` (done) vs `[cut]` (deferred to Phase 2)?
- **For verify:** verdict + commit SHA.

### Cost self-report
- **Tokens (total):** <real>  **Estimated USD:** <n>  **Duration (min):** <n>  **Source:** `/cost` | transcript | harness

### Drift and new artifacts
- **New decisions:** <DEC-NNN or none>  **Deviations:** <list>  **Follow-ups:** <list>

### Reflection
1. What was unclear? — <answer>
2. A constraint/decision that should have been listed? — <answer>
3. What would you do differently? — <answer>
