# SPEC-011 — build dispatch

Build cycle for **SPEC-011: lint the fuzz crate — the second cargo graph sits
outside every -D warnings gate** (STAGE-005, PROJ-001). Delegated via **HANDOFF-060**.

Read first, in order:
1. This prompt.
2. `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-060-build-lint-the-fuzz-crate-the-second-cargo-graph-sits-outside-every-d-warnings-gate.md`.
3. The spec (SPEC-011) — Inputs (file map), AC1–AC5, Notes (key facts).
4. `guidance/toolchain-brief.md`.

Add a clippy gate for the `fuzz/` crate (linted by nothing today, because DEC-011
keeps it as a separate cargo graph). Four traps:

1. **No code fixes — the fuzz crate is ALREADY clippy-clean** (0 warnings on CI
   clippy 0.1.98 and 0.1.99, even under the five panic-free lints). The gate ships
   green immediately. So the **red-proof's injected fault must be a DEFAULT-clippy
   violation** (an unused variable/import, `needless_return`, etc.) — NOT a
   panic-free-only one (those aren't in scope and the crate passes them).
2. **Scope decision → DEC-027**: the fuzz crate gets clippy **default `-D
   warnings`, NOT the five panic-free lints** (dev-only glue; a harness panic is a
   test bug libFuzzer catches, not a product vuln; precedent: `irr`/`#[cfg(test)]`
   exemptions + the `--lib` scope of `no-panics-on-untrusted-input`). Record it in
   DEC-027 extending DEC-011's "two invocations" principle to clippy; weigh the
   counter-argument; regenerate INDEX; keep `decisions-audit` clean.
3. **Red-proof** (`scripts/lint-fuzz-red-proof.sh`, modeled on
   `scripts/lint-red-proof.sh`/DEC-009): negative control (unmutated fuzz copy
   passes `-D warnings`) + an injected default-clippy fault caught, name-matched
   robustly (clippy help-URL fragment, not the bare name), temp copy only — working
   tree stays clean. Plus its CI job.
4. **Clippy pinning** (PATCH-004, 17 red runs): CI job uses `@stable`; state the
   local recipe's pinning. Mirror `deny-fuzz`/`licenses-fuzz`; no nightly. Add both
   recipes to AGENTS.md §6's block.

Fill the handback with a REAL transcript-sum `tokens_total` (deduped by
message.id via your scratchpad UUID — NOT a remaining-budget delta). Open a PR
against `main`; confirm CI green by observation. zsh: `$pipestatus`.
