# SPEC-039 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-039-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-03) — original uncompressed-only plan. Superseded: the pre-0.2.0
  review (finding #5) showed it does not carry to Compression-7, so the spec was
  RE-SCOPED to group E (DEC-045) and re-designed after A–C shipped.
- [x] **design** (orchestrator, 2026-11-10) — re-designed against the current tree after
  A/B/C landed. DEC-050 minted: one public `plane::validate(sensor, byte_order, file,
  max_pixels)` covering both the uncompressed and Compression-7 paths, with a
  caller-selected pixel budget (`Error::PlaneTooLarge`) checked first as the universal
  pre-allocation guard (the uncompressed file-bytes bound `StripBytesExceedFile` does not
  hold for compressed), the shared-helper refactor of `unpack_into` preserving error
  precedence, the EOI-optional ratification (grounded: the decoder completes on sample
  count, not EOI), the `ifd`/`plane`/`ljpeg` fuzz smoke jobs, and the README/CHANGELOG/
  conformance-matrix reconcile + a tested-DNG-modes compatibility table. Group C's two
  verify follow-ups were deliberately deferred (not folded in — they re-touch C's opcode
  path). `HANDOFF-084` superseded by `HANDOFF-117` (build). Spec moved `frame → build`.
- [~] **build** — delegated via HANDOFF-117. Branch `claude/spec-039-build-<suffix>` from
  freshly-merged `main`. Build exactly DEC-050; red-proofs shown red first (coverage,
  aliasing, budget); fuzz; tier-B corpus bit-exact + trips neither new error (local run);
  gates; PR; fill the handback.
- [ ] **verify** — cold review: the budget check fires first on both paths, `validate` and
  `unpack_into` share the checks (no drift), the uncompressed bound holds under aliased
  strips, Compression-7 validates structurally and the shape binding still fires at decode,
  EOI-optional holds, the corpus is bit-exact, and the docs match the code.
- [ ] **ship** — merge, recover true build/verify cost, brag, stage bookkeeping.
