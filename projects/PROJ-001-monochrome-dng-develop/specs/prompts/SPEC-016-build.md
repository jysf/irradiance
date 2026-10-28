# SPEC-016 — build dispatch

Build cycle for **SPEC-016: the harness stops claiming what it has not checked**
(STAGE-005, PROJ-001). Delegated via **HANDOFF-056**.

⚠ Do NOT follow `HANDOFF-038` — it is the stale L-scope (five ACs). This is the
S-scope, one deliverable: `just validate` actually PARSES front matter.

Read first, in order:
1. `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-056-build-the-harness-stops-claiming-what-it-has-not-checked.md`
2. The spec — especially **Design refresh (2026-10-27)** (current numbers),
   **AC1–AC3**, **Failing Tests**, **Traps**.
3. `guidance/toolchain-brief.md`.

The task: rewrite `scripts/validate.sh` to parse every artifact's front matter
(under `projects/**`, `decisions/`, `handoffs/`, `patches/`) with a real YAML
parser, failing loudly with the file + the parser's own message, keeping the
existing required-keys check. No `src/` change. Four traps that sink this build:

1. **Use a parser already present on CI.** `ruby -ryaml` is on macOS + GitHub
   runners; `python3` now *also* has `pyyaml` here, but confirm whatever you
   pick exists on the CI image (AC1). Don't add a dep, a Rust binary, or a
   container.
2. **The red-proof must show the parser's MESSAGE, not just an exit code.** A
   wrong parser API fails *uniformly* (e.g. `safe_load(permitted_classes:)` on
   Ruby 2.6 raises `ArgumentError` on every file) — that looks like a finding
   but is a broken check, the exact class this spec closes. Show OLD-passes /
   NEW-fails on the same fixture bytes (AC2, DEC-017 mechanism).
3. **Broadening scope reveals one real break: `HANDOFF-039`** (front matter
   unparseable, `did not find expected key` ~line 49). Fix its front matter so
   the new gate ships green — that fix IS the gate doing its job. Report the
   cause and the post-fix count (expect 0 of 120).
4. **`just lint-ci` (pinned clippy), read CI, name the gate list you ran**
   (the count is ambiguous in this repo — AC3 works around it). zsh: use
   `$pipestatus`. Fill the handback with a REAL `tokens_total` before reporting
   done; open a PR against `main`.
