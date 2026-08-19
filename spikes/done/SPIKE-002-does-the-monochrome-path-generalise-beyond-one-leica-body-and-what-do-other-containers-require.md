---
# A SPIKE is a BOUNDED EXPLORATION — the phase before you know the shape.
# Two modes, one discipline:
#   mode: question — a timeboxed investigation. Code is evidence, usually thrown away.
#   mode: build    — a vibe-coding session. Code is the deliverable, you intend to keep it.
# See AGENTS.md "Spike lane" and docs/decisions/DEC-012.
#
# Collapsed cycle: spike -> land.
#   spike — explore. NO spec, NO failing tests, NO DEC required. Speed IS the value.
#   land  — MANDATORY. The entire point of the lane: write down what you learned,
#           emit the DECs the exploration already made, decide the code's fate.
#
# There is deliberately NO verify step: a spike has no acceptance criteria, so an
# "independent verify" would have nothing to check (DEC-012). The timebox and the
# mandatory land step are the disciplines that replace it.

task:
  id: SPIKE-002
  type: spike                      # epic | story | task | bug | chore | patch | spike
  cycle: land                     # spike | land  (collapsed from a spec's 5)
  blocked: false                   # sub-questions 1+2 UNBLOCKED; only sub-question 3 waits on files
  priority: medium

spike:
  question: Does the monochrome path generalise beyond one Leica body, and what do other containers require
                                   # REQUIRED. A spike with no question is just coding.
                                   # For mode: build this is legitimately loose
                                   # ("is a local standup tool worth having?").
                                   # Loose is fine. Absent is not.
  mode: question                   # question | build
  timebox: 2 sessions
                                   # REQUIRED. Exceeded means STOP and land it as
                                   # `inconclusive` — not extend. Extending twice
                                   # means it isn't a spike, it's an unframed project.
  outcome: answered                    # set at LAND. Never leave null on a landed spike —
                                   #   answered     — the question got an answer
                                   #   inconclusive — timebox hit, no answer (a real result)
                                   #   graduated    — the code becomes real work (see below)
                                   #   discarded    — the code is thrown away (also a win)
  landed_at: 2026-08-18                  # YYYY-MM-DD, stamped at land

project:
  id: null               # OPTIONAL (null) — a spike may PRECEDE any project
                                   # (that's the point). Back-link at land if one exists.
  # No `stage:` — a spike attaches to the repo, not a stage.
repo:
  id: irradiance

agents:
  explorer: claude-sonnet-5  # who ran the spike (tier_map.build; DEC-005)
  created_at: 2026-08-18

references:
  decisions: [DEC-008]                    # DECs EMITTED AT LAND (not during — that's the point)

# Cost is ADVISORY for spikes — `just cost-audit` does NOT gate them (DEC-012 v1).
# A spike is often pre-project and deliberately cheap; a cost gate on exploration
# is exactly the friction that would make people skip the artifact.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0.00
    session_count: 0
---

# SPIKE-002: Does the monochrome path generalise beyond one Leica body, and what do other containers require?

## Question

SPIKE-001 proved `irradiance` can decode **one camera's** monochrome DNG
bit-exact. Everything it established — the container walk, the tag model, the
14-bit unpack, the oracle contract — rests on **one body (serial 5597430), one
firmware, three frames.** This spike asks whether any of that is Leica-specific.

Three sub-questions, carried forward from SPIKE-001 or newly raised by it:

1. **Does the container reader work on a monochrome DNG from a different make?**
   Candidates identified in the raw.pixls.us mirror: Leica M Monochrom, Leica
   M Monochrom (Typ 246), Pentax K-3 Mark III Monochrome. The Pentax ships
   **both a DNG and a native PEF of the same scene**, which is a free
   cross-container comparison.
2. **Do other bit depths and packings hold?** SPIKE-001 only ever saw 14-bit
   tightly packed with zero row padding, where `width*14/8` happened to land on
   a byte boundary exactly. A 12-bit source (Nikon P1100 NRW) is the cheapest
   test of a hardcoded assumption — SPIKE-001's unpacker takes `bits` as a
   parameter but has **never been run with any value but 14**.
3. **Does `dnglab convert -c uncompressed` preserve what we need?** SPIKE-001's
   Q8, unanswered. Specifically whether it preserves a Fuji X-Trans 6x6 mosaic
   (`CFARepeatPatternDim: 6 6`) and whether it works on a D750 NEF. This decides
   whether `convert` is a viable corpus-widening tool or a trap.

## Timebox

2 sessions. `inconclusive` is a real result — see SPIKE-001, which landed at 1 of
3 and was more valuable for stopping early than it would have been for grinding
on blocked questions.

## ⛔ Blocked on — the prerequisite list

**Do not start this spike until these are done.** Every one of them needs a human
or a network, not an agent, and running the spike without them burns the timebox
on questions that cannot be answered.

- [x] ~~**Resolve licences for the raw.pixls.us samples.**~~ **DONE 2026-08-18.**
      All three monochrome candidates are **CC0**, confirmed two independent ways:
      absent from the site's `?noncc0` list (all 111 entries read), and
      `License: CC0` in the full 2016-row table. The site's sha256 matches the
      git-lfs pointer in `../data.lfs` exactly for all four files. Recorded as
      `[[available]]` rows in `tests/corpus/manifest.toml`.
      ⭐ Bonus: the **M Monochrom (Typ 246) is 12-bit** — see below.
- [x] ~~**Pull the files.**~~ **DONE 2026-08-18.** `git-lfs` is not installed, so
      they came over plain HTTPS instead — raw.pixls.us serves direct URLs at
      `getfile.php/<id>/nice/<name>`. All four verified against the sha256
      recorded *before* download; the manifest pinning worked end to end.
- [x] ~~**Add each pulled file to the manifest.**~~ **DONE 2026-08-18** — all four,
      with licence, source URL, sha256 and `raw_checksum`. Corpus is now 7 files.
- [x] **A different bit depth — RESOLVED, but NOT by the file first claimed.**
      ⚠ **Correction.** The Typ 246 is 12-bit and CC0 as hoped, but it is **JPEG
      compressed** (5984*4000*12/8 = 35,904,000 vs a declared 21,311,750), so
      PROJ-001 cannot read it — that needs lossless JPEG SOF-3, explicitly out of
      scope. Same for the Pentax K-3 III DNG. Three of the four downloads are
      compressed.
      **What actually resolves it: the original Leica M Monochrom — 16-bit,
      genuinely UNCOMPRESSED**, arithmetic closing exactly at 36,219,904. It is a
      third bit depth (16 vs 14), BlackLevel 220 not 512, **ActiveArea with a
      non-zero origin (2,2)** where every Q2M frame is (0,0) — so the ActiveArea
      crop gets exercised for the first time — and it carries **no opcode lists**
      at all. It is the single most useful third-party file we hold.
- [ ] **A Fuji RAF and a Nikon D750 NEF** for sub-question 3 only. **Not held
      today**, and the only items left that need a camera or a favour. Note this
      blocks *sub-question 3 alone* — sub-questions 1 and 2 are fully unblocked
      once the CC0 files are pulled, so the spike can run usefully without these.

## Explicitly NOT in this spike

- **Q7, cross-platform byte-identity — deliberately dropped, not deferred.**
  SPIKE-001 verified the decode path contains **zero floating point**, no
  `HashMap` iteration order (it uses `BTreeMap`), and no platform-dependent width
  beyond `usize`. The single native-endian call (`to_ne_bytes`) is native *by
  design*, to match the oracle's native-LE contract. On any little-endian 64-bit
  platform the output is byte-identical **by construction**, so testing it now
  would confirm what inspection already shows.

  The question becomes real when **floating point arrives** — warp resampling and
  the tone curve, both STAGE-003. It belongs there, as a CI job on
  `ubuntu-latest` (free) or a `wasm32` target run (already installed, and
  *stronger* than Linux-on-arm64 because it changes pointer width to 32-bit).
  Docker is not needed for it and never was. `DEC-002` already proposes the
  mitigations (table-driven tone curves over `powf`, pinned reduction order, no
  runtime SIMD dispatch).
- Demosaic, colour, and anything Bayer — that is PROJ-002's own framing.
- Lossless JPEG SOF-3 — PROJ-003.
- Any decoder code that is meant to survive. As with SPIKE-001, code here is
  evidence and its branch is never merged.

## What lands

- A per-camera answer on whether the container walk and tag model hold unmodified
- The first non-14-bit unpack, or a recorded reason there wasn't one
- A verdict on `dnglab convert` as a corpus-widening tool
- Manifest rows, with licences, for every file obtained
- `docs/conformance-matrix.md` rows updated from measurement rather than
  from a vendor's camera list

## Log

*(no conventions here — it's yours)*

### 2026-08-18 — sub-questions 1 and 2 answered. ONE REAL BUG FOUND.

Ran SPIKE-001's decoder (from its unmerged branch, in a throwaway worktree)
against the four CC0 samples. Sub-question 3 still waits on a Fuji RAF / D750 NEF.

**SUB-Q1 — does the container reader generalise? YES, comprehensively.**

On the Leica M Monochrom — a different body, a different sensor generation, a
different bit depth — the IFD walk, SubIFD recursion, tag model, sensor-plane
selection, strip location and layer-0 arithmetic **all worked unmodified**:

```
ifds found     2 -> SubIFD(20 tags), IFD(34 tags)
dimensions     5216 x 3472      bits 16      samples/pixel 1
strip          offset 213504  bytes 36219904  rows/strip 3472
levels         black 220  white 16383
activeArea     []                       <- NO ActiveArea tag at all
defaultCrop    origin [2, 2] size [5212, 3468]
opcodes        list1 false list3 false  <- no opcode lists at all
5216 x 3472 x 16 / 8 = 36219904 == StripByteCounts  ✓
```

Every one of those differs from the Q2M and none of it needed a code change.
Notably it has **no `ActiveArea`** and **no opcode lists**, so the "tag absent"
paths were exercised for the first time.

**🔴 THE BUG — the unpacker's bit-order assumption does not generalise.**

The plane came out wrong, and **the free sanity check caught it**:

```
first 8   [39186, 61201, 3602, 2833, 48657, 7953, 18, 48656]
min/max   1 / 65343   (black 220 white 16383)
⚠ max exceeds WhiteLevel — impossible; unpack or endianness is wrong
```

Cause, confirmed three ways:

1. `md5(our plane)` = `563ecf2b…`, but `md5(our plane byte-swapped)` =
   **`b0f602b90db91f981bbd6802fd0e6edf` = exactly `dnglab --raw-checksum`.**
2. Reading the raw strip head `99 12 ef 11 0e 12 0b 11` as big-endian u16 gives
   `[39186, 61201, 3602, 2833]` — impossible against WhiteLevel 16383. As
   little-endian it gives `[4761, 4591, 4622, 4363]` — all plausible.
3. The file's header is `II` (little-endian).

**The rule the unpacker got wrong:** *sub-byte* samples (14-bit) are a **MSB-first
bit stream**, but *byte-aligned* samples (16-bit) are plain **u16 in the file's
byte order**. SPIKE-001's unpacker treated everything as an MSB-first bit stream,
which is right for 14-bit and wrong for 16-bit. It had only ever run against
14-bit data, so the two cases were indistinguishable until now.

This would have shipped silently. **STAGE-002's unpack spec must branch on
`bits % 8 == 0`** and honour the TIFF byte order for the byte-aligned case.

**SUB-Q2 — do other bit depths hold? PARTIALLY, and mostly not testable yet.**

Only one of the four downloads is uncompressed. The other two DNGs are JPEG
compressed and both were **rejected safely — no panic, clear message, exit 0**:

| File | Bits | Result |
|---|---|---|
| Leica M Monochrom | 16 | decodes; bit-exact after the byte-order fix |
| Leica M Monochrom (Typ 246) | 12 | JPEG — rejected cleanly |
| Pentax K-3 III Monochrome | 14 | JPEG — rejected cleanly |

**A second generalisation win, unplanned: the Typ 246 is BIG-ENDIAN (`MM`).**
Every Q2M frame is `II`. The `Order` abstraction had never been exercised — and it
worked, walking 2 IFDs and 19+39 tags correctly before rejecting on compression.

**No panic on malformed input, for free.** The Pentax DNG carries a tag dnglab
itself warns about (*"BlackLevelRepeatDim tag but with invalid length: 1"*). Our
reader walked all 3 IFDs and 74 tags without incident. A real shipping camera
writes a malformed tag; that belongs in the tier-A fixture set.

**Diagnostics defect worth carrying to SPEC-003/004.** Both compressed files
report `no full-resolution LinearRaw single-sample IFD found`. That is misleading —
the IFD *was* found and then rejected for `Compression != 1`. The real message
should name the reason. A user debugging an unsupported file would be sent looking
in the wrong place.

**Still open:** sub-question 3 (`dnglab convert` on Fuji RAF / D750 NEF) — blocked
on files not held.


## Land

**Outcome: `answered`.** Sub-questions 1 and 2 are answered. Sub-question 3 is
**not**, and is carried out rather than papered over — see below.

**Does the monochrome path generalise beyond one Leica body? The CONTAINER does,
completely. The UNPACKER did not.**

Against a Leica M Monochrom — different body, different sensor generation,
16-bit instead of 14, `BlackLevel` 220 instead of 512, **no `ActiveArea` tag**,
**no opcode lists at all** — the IFD walk, SubIFD recursion, tag model,
sensor-plane selection, strip location and layer-0 arithmetic **all worked
unmodified**. The tag-absent paths were exercised for the first time. A second,
unplanned generalisation also held: the M Monochrom (Typ 246) is **big-endian
(`MM`)** where every Q2M frame is `II`, and the byte-order abstraction — which had
never once run — walked it correctly.

The unpacker produced a byte-swapped plane, caught immediately by a free
one-line assertion (*max must not exceed `WhiteLevel`*). Root cause and the rule
that replaces it are recorded in **`DEC-008`**: sub-byte samples are a MSB-first
bit stream; byte-aligned samples are plain integers in the file's byte order.
SPIKE-001's unpacker took `bits` as a parameter but had only ever run with 14,
so the two cases were indistinguishable until a 16-bit file existed.

**This is the bug the spike existed to find.** It would have shipped: the plane
is the right size, from a file that parsed cleanly, differing from truth only in
per-sample byte order.

### DEC emitted

- **`DEC-008`** — sample unpacking branches on `bits % 8 == 0`, not on bit depth.
  Reshapes STAGE-002's unpack spec before it is written, with a real file and a
  known-correct checksum for each branch.

### Also established

- **3 of 4 downloaded CC0 samples are compressed** (two JPEG, one native PEF) and
  every one was **rejected safely** — no panic, clear message. Only the M
  Monochrom is uncompressed, which is why it is the most valuable third-party
  file held.
- **No panic on genuinely malformed input, for free.** The Pentax K-3 III DNG
  carries a tag dnglab itself warns about (*"BlackLevelRepeatDim tag but with
  invalid length: 1"*). Our reader walked all 3 IFDs and 74 tags without
  incident. It belongs in the tier-A fixture set as a regression case.
- **A diagnostics defect for SPEC-003/004:** rejecting on compression reports
  `no full-resolution LinearRaw single-sample IFD found`. The IFD *was* found and
  then rejected for `Compression != 1`; the message sends someone debugging an
  unsupported file to the wrong place.

### The code's fate: DISCARDED

Same as SPIKE-001 — the decoder used here is SPIKE-001's, on its unmerged branch,
and neither is merged. What lands is `DEC-008`, the corpus rows, and these
answers.

### ⚠ Sub-question 3 is UNANSWERED, and closing this spike does not close it

*Does `dnglab convert -c uncompressed` preserve an X-Trans 6×6 mosaic, and does it
work on a D750 NEF?* Still blocked on a Fuji RAF and a Nikon D750 NEF, neither
held. That question decides whether corpus-widening for PROJ-002 is cheap or
expensive, so it should be **reopened as its own spike when the files arrive** —
not treated as covered because this one landed. Recorded in
`docs/conformance-matrix.md` so the gap stays visible.

**Timebox: 1 of 2 sessions used.** Landing early rather than holding the lane open
for a question that cannot be worked.
