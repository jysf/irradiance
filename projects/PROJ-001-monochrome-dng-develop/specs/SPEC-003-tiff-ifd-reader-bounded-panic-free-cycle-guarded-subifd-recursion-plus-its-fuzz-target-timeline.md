# SPEC-003 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-003-<cycle>.md`.

## Instructions

- `[x]` **design** — 2026-08-20, `ee5f310`. Designed the reader and, more
  usefully, **found and fixed the `cargo fuzz` blocker before build started**:
  the inner bare `"cargo" "build"` resolves to Homebrew's stable cargo and
  rejects `-Zsanitizer`, so the shim must be first on `PATH`. Proven end to end
  (init works, 32.9 M execs in 16 s, a planted unchecked index caught with exit
  77), which is why acceptance criteria 4 and 5 were known-achievable rather
  than hoped for.
- `[x]` **build** — 2026-08-20, `b79c7ef` on `feat/spec-003-ifd-reader`, not
  merged. HANDOFF-011. All 7 acceptance criteria met; nine gates green; both
  fuzz directions pasted in the handback. No `#[allow]` of any policy lint was
  needed. New: `DEC-011`. Two measured corrections to the spec's own notes —
  only **one** corpus file is big-endian, and `K3III.PEF` has no `SubIFD` at
  all (its plane is in IFD0, with no `NewSubfileType` tag). 10,967,269 tokens,
  deduped by `message.id`.
- `[ ]` **verify** — HANDOFF to be written by the architect.
