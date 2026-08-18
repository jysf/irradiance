---
insight:
  id: DEC-003
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-16
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# NB on numbering: DEC-001 is deliberately vacant. It was occupied by the
# template's seeded example decision, deleted 2026-08-16 (see DEC-000). IDs are
# never reused, so the gap stays.

affected_scope:
  - tests/corpus/**
  - docs/conformance-matrix.md

tags:
  - corpus
  - testing
  - licensing
  - provenance
---

# DEC-003: The tier-B corpus lives outside the repo; the manifest lives inside it

## Decision

**Real camera files are never committed — not raw, not via git-lfs, not cropped.**
They live in a directory outside the repo, located at run time by
`$IRRADIANCE_CORPUS_DIR`. What *is* committed is
**`tests/corpus/manifest.toml`**: one entry per file carrying its relative path,
size, `sha256`, make/model, **licence**, **source**, and the **pinned oracle
answers** (`dnglab analyze --raw-checksum`, PGM byte count, `StripByteCounts`).

Two consequences follow, and both are rules:

1. **Tier A (committed, runs in CI) admits only `CC0` or own-work.** Anything
   carrying attribution, ShareAlike or NonCommercial terms is tier B forever.
2. **Tier-B tests skip loudly**, naming the missing file. A silent skip reports
   green for work it never did.

## Context

`docs/conformance-matrix.md` said *"Decide storage (git-lfs vs fetch-on-demand)
before the first fixture lands."* Files start arriving today, so the decision is
due now — after it, every choice is a migration instead of a decision.

Three facts force the shape:

- **Size.** A Q2M frame is ~86 MB. Git stores every version forever; a handful of
  frames is a repo nobody wants to clone, and this library's whole point is being
  easy for others to adopt.
- **Licence.** Verified 2026-08-16: the obtainable third-party samples are *not*
  freely redistributable. [raw.pixls.us](https://raw.pixls.us/) is mixed — CC0
  preferred for new submissions, but rawsamples.ch imports are **CC-BY-NC-SA**.
  The one Q2 Monochrom file found from a second body is **CC BY-SA 4.0**. Review
  sites are "personal evaluation only". **Local use is not redistribution** — but
  committing is.
- **Provenance is unreconstructable.** A file whose licence was not recorded when
  it arrived is a file that can never be confidently used or shared again. Unlike
  most defects there is no back-fill. This is AGENTS.md §11's "ship the reader
  with the field" in its most expensive form, which is why the manifest is being
  seeded *before* the files land rather than after.

## Alternatives Considered

- **Option A: git-lfs.**
  - Why rejected: it does not solve the licence problem at all — an LFS-tracked
    CC-BY-NC-SA file is still redistributed by the repo. It also pushes bandwidth
    and an LFS dependency onto every consumer of a library whose selling point is
    being frictionless to adopt, and the objects are in history permanently.

- **Option B: commit small crops of real files.**
  - Why rejected: a crop is a *different file*. It changes `StripByteCounts`,
    `ActiveArea`, `DefaultCropSize` and the plane checksum — precisely the values
    under test. It would verify a fiction, convincingly.

- **Option C: fetch-on-demand script pulling a CC0 subset.**
  - Why rejected **for now, not on the merits**: it is the right eventual answer
    and composes with this decision (the manifest already carries the hashes a
    fetcher needs). But there are **zero** CC0 files in the corpus today, and the
    only blocking need is the maintainer's own frames, which no script can fetch.
    Build it when the manifest has CC0 rows worth automating.

- **Option D (chosen): external directory + committed manifest.**
  - Why selected: it is the only option that keeps the repo small, keeps the
    licence surface clean, and still pins exactly what each test expects. The
    manifest is the artifact that makes an absent corpus *legible* rather than
    invisible.

## Consequences

- **Positive.** No RAW file can enter git history. Every file's licence is
  answerable from one committed file. The pinned `raw_checksum` means a dnglab
  version bump that changes the oracle's answer is **caught**, rather than
  silently redefining ground truth. Declared-`wanted` rows keep the gaps visible.

- **Negative — and this is the real cost. CI cannot verify bit-exactness.**
  Tier B is absent on a CI runner, so the bit-exact plane oracle — the project's
  central claim — runs **only on a machine holding the corpus**. CI is reduced to
  tier A, fuzz, lint and the language-agnostic gates. This is accepted because the
  alternative is unlicensed redistribution, but it must not be papered over: a
  green CI badge on this repo does **not** mean the decoder is bit-exact. Say so
  in the README when one exists.

- **Negative.** A new contributor must set an env var before any tier-B test does
  anything, and the skip message is the only thing that will tell them.

- **Neutral.** `IRRADIANCE_CORPUS_DIR` defaults to the repo-local
  `tests/corpus/tier-b/` (already `.gitignore`d), so the external directory is an
  override rather than a requirement.

## Validation

Right if, at PROJ-001's close:

- `git log --stat` shows **zero** files with a RAW extension, ever.
- Every file in the corpus has a licence and a source in the manifest — checked by
  reading it, not by remembering.
- At least one pinned `raw_checksum` mismatch has been *caught* by the manifest
  (a dnglab upgrade, a re-download, a truncated copy). If nothing is ever caught,
  the pinning is decoration and should be justified or dropped.

Revisit if:

- The corpus acquires enough CC0 files that fetch-on-demand (Option C) would
  meaningfully improve reproducibility — the likely trigger is PROJ-002 pulling
  Bayer samples from raw.pixls.us.
- The CI gap starts hiding real regressions. The fix then is a self-hosted or
  scheduled runner with the corpus mounted, **not** committing files.

## References

- Corpus policy and the tier split: `docs/conformance-matrix.md`
- The manifest itself: `tests/corpus/manifest.toml`
- Oracle answers being pinned: `docs/oracle-contract.md`
- Instantiation record, incl. the vacant DEC-001: `decisions/DEC-000-template-instantiation.md`
- Constraints: `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`
- Sample sources checked 2026-08-16: [raw.pixls.us](https://raw.pixls.us/) (per-file licence, CC0 filter, git-lfs/git-annex bulk access) · [rawsamples.ch legal terms](https://www.rawsamples.ch/index.php/en/legal-stuff) (CC-BY-NC-SA)
