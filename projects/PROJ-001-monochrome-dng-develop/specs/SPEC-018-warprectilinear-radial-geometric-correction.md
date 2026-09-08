---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-018
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: critical               # critical | high | medium | low
                                   # ⚠ RAISED from the frame stub's `medium`.
                                   # SPIKE-001 measured ~504 px of corner
                                   # displacement — 6 % of image width. This is
                                   # the single item in PROJ-001 that most
                                   # decides whether the thesis holds: the
                                   # library today produces an image visibly
                                   # wrong by 6 % at the corners, and no
                                   # reference render will ever match. STAGE-003
                                   # marks it NOT DEFERRABLE for the same reason.
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
                                   #   L matches the frame. Two surfaces:
                                   #   (i) OpcodeList3 parser (new input surface,
                                   #   panic-free-on-untrusted-input, its own
                                   #   fuzz target — §12 bar 2 fires); (ii) the
                                   #   radial warp resampler and its integration
                                   #   into develop_into. Kept a single spec
                                   #   because the resampler is coupled to the
                                   #   opcode coefficients: splitting risks a
                                   #   parser that ships without a caller and
                                   #   the read-with-the-field failure mode
                                   #   (AGENTS.md §11).
  complexity_actual: XL            # XL — expected L, shipped XL. Three build rounds (116.5M + 14.3M + 5.6M = 136.4M) and three verify rounds (22.6M + 6.8M + 3.3M = 32.7M), totalling 169M tokens across 6 sessions vs L-oracle-shape SPEC-015's 98M. The overrun came from (a) Finding 2 (dnglab doesn't apply opcodes, invalidating SPEC-020's oracle for this stage) landing at round-1 build's SSIMULACRA2 measurement — an unpredictable structural discovery, not scope creep; and (b) two punch-list rounds closing SB-1 (missing live test), SB-2 (false rustdoc claims), and SB-3 (a false citation introduced by the SB-2 fix itself — `a-fix-inherits-the-precondition`). If the oracle-broken discovery is treated as a "shipped a real finding" rather than "overran budget", the size is L. Recorded as XL because complexity_actual is what it took, not what it was worth. stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-002, DEC-004, DEC-005, DEC-011, DEC-016, DEC-018]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, library-not-application, test-before-implementation]
  related_specs: [SPEC-003, SPEC-014, SPEC-015, SPEC-020]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-020]             # SPEC-018 is scored through SPEC-020's oracle
                                   # (AC10). Building SPEC-018 without SPEC-020
                                   # is designing the render against its own
                                   # success — the trap SPEC-015 was created to
                                   # break, applied here to STAGE-003.

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-003's <capability>". Optional; null is acceptable.
value_link: "closes the ~504 px corner error SPIKE-001 measured — the single
  item in PROJ-001 that most decides whether the thesis holds. Missing it, the
  library produces an image visibly wrong by 6 % at the corners and no
  reference render matches."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 130000000
  # Basis: SPEC-014 (levels + geometry, expected L, shipped) landed at
  # ~88.8M; SPEC-015 (oracle, expected M, shipped L) landed at ~98.2M; both
  # attracted a punch-list round. SPEC-018 has TWO new surfaces (an opcode
  # parser AND a resampling kernel), a mandatory fuzz target (§12 bar 2 —
  # new input surface), integration into develop_into, and the resampling
  # kernel is a design choice with visible calibration cost. 130M assumes
  # build + verify + one punch-list round; the parser adds a fuzz seed
  # session too. If it lands near 90M the two surfaces overlapped more than
  # expected; near 180M and STAGE-003's remaining L (a hypothetical) should
  # be re-framed as a stage-of-its-own.
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 116480125
      estimated_usd: 49.61
      duration_minutes: 82
      recorded_at: 2026-09-07
      notes: 12/14 ACs green; AC8/AC9
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 22631969
      estimated_usd: 51.98
      duration_minutes: 22
      recorded_at: 2026-09-07
      notes: PUNCH LIST on 40f5d45 (CI green there and on tip 823a7fc, 10 jobs each). Finding 1 is spec-correct but UNTESTED - reverting develop.rs to crop-then-warp compiles, changes real output, and leaves the whole suite green at 205/0/2, so the warp branch of develop_into has zero live coverage (SB-1). Finding 2 CONFIRMED behaviourally without reading dnglab source (corner-tile NCC vs dnglab - 0.989 to 0.995 for our UNWARPED render, minus 0.31 to plus 0.23 for our warped one, on two frames) and judged FU-10, ship with the ignore-marked tests. AC8 minus 60.169 and AC9 minus 60.193 / minus 55.075 reproduced exactly. SB-2 is two false claims in shipped rustdoc. Nine further follow-ups FU-1..FU-9.
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 14298209
      estimated_usd: 34.26
      duration_minutes: 29
      recorded_at: 2026-09-07
      notes: "Round 2 (punch-list) on HANDOFF-047's two ship blockers, both CLOSED at 08ad42e. Docs and tests only - no logic changed, decoded output byte-identical (irr develop L1021223.DNG samples[0..8] and max 51764 unchanged). SB-1: new tier-A test develop_into_crops_from_the_warped_active_area_not_the_warped_crop is the only test in the tree that develops a non-None opcode_list_3 all the way to pixels; hand-built Sensor whose ActiveArea 80x64 and DefaultCrop 60x48 at origin (7,5) genuinely differ, real L1021223 OpcodeList3 bytes, affine ramp plane. Three assertions - the fixture separates the two orders (2837 of 2880 pixels, asserted), develop_into equals crop(warp(active)) and not warp(crop(active)) whole-buffer plus two probe pixels, and an identity warp develops bit-identically to no opcode list while differing from the real warp. Red-proof observed both directions: src/develop.rs md5 318977b683d168d4572efdd5f672cc98 honest -> cf714e8c973c5b4c60c647cca181ca77 crop-then-warp -> 318977b683d168d4572efdd5f672cc98 reverted; the mutation compiles and moves real output (max 51764 -> 60918), and the suite under it is 205 passed / 1 FAILED / 2 ignored - round 1's 205 all stayed green, which is SB-1 restated as a measurement. SB-2: both false rustdoc claims rewritten citing DEC-024 - src/warp.rs no longer claims three frames scored at or above 85 through an oracle that cannot see this stage (DEC-024 Finding 2), and now rests the kernel choice on AC4/AC10, whose 339.5 px separation I re-measured by running the test rather than copying it; src/lib.rs no longer states the pre-Finding-1 pipeline order it introduced in 40f5d45, the very commit that corrected the code (verified with git show). Incidental, in the same doc comments: three dangling kernel-choice DEC-* placeholders resolved to DEC-024, and one of them exposed a further false claim of the same species - DEC-024 records no third-caller-supplied-buffer alternative - now corrected to cite what it does hold. Gates on 08ad42e: fmt, lint-ci on the PINNED clippy 0.1.98 (not local 0.1.97), typecheck, test at 206 passed / 0 failed / 2 ignored (+1, as predicted), deny and deny-fuzz, msrv 1.90.0, lint-no-allow, both red-proof scripts, and fuzz-warp at 22,937,786 runs in 61 s with zero crashes. CI GREEN on 08ad42e - run 34164609782, 10 jobs all success, fuzz smoke warp_opcode included. FU-1 through FU-10 untouched and still owed a disposition at ship; DEC-024, the ACs and PR 16 all untouched."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 6790688
      estimated_usd: 17.38
      duration_minutes: 16
      recorded_at: 2026-09-07
      notes: "PUNCH LIST (round 3) on 08ad42e - CI green there, run 34164609782, 10 jobs all success including the warp_opcode fuzz smoke. SB-1 CLOSED. Ran the test with --exact: exactly one match, passed, and the fixture prints 2837 of 2880 developed pixels separating the two orders, reproducing the build claim exactly. Reproduced the red-proof myself on the SHIPPED file: src/develop.rs md5 11118242d58a04ce60351593b2e437f1 honest -> f620d615c80da5474f46bb9d11f7ec44 crop-then-warp -> 11118242d58a04ce60351593b2e437f1 reverted byte-identical; the mutation compiles and moves real output (irr develop L1021223.DNG samples[0..8] [2028,1855,1818,1868,1805,1889,1943,2013] max 51764 -> [1801,2001,2036,1972,1819,1689,2142,2185] max 60918), suite 206/0/2 -> 205 passed / 1 FAILED / 2 ignored with the one failure being the new test at develop_into = 4130 vs warped-ActiveArea 3948, and round 1s 205 all green under it. Note the build handbacks md5 pair 318977b6 -> cf714e8c is src/develop.rs at 40f5d45, i.e. before round 2s own rustdoc edit to that file, so the shipped file was never itself the subject of the recorded pair - my pair above is on the shipped file and is the one that satisfies DEC-004 rule 1. SB-2 CLOSED for both claims it named: src/warp.rs Kernel section no longer fabricates >=85 per-frame scores and now rests the choice on AC4/AC10 citing DEC-024 Finding 2 (AC10 re-run here: 339.5 px against the 20 px bar; AC4 re-run: 503.7 / 437.7 / 407.0 px per-frame, within 1 px of SPIKE-001), and src/lib.rs 56-62 now states the shipped order - after normalize, over the full ActiveArea, before DefaultCrop and Orientation - citing DEC-024 Finding 1, verified against src/develop.rs 575-613 and against git show 40f5d45 for its self-reference. The incidental third-caller-supplied-buffer correction is HONEST: DEC-024 Alternatives records only Options A, B and C (kernel and threshold), no caller-supplied-buffer option, and its Consequences does record the two ActiveArea-sized scratch buffers at 465,010,688 bytes peak RSS, matching docs/provenance-ledger.md src/warp.rs row. NEW: SB-3, ship-blocking, same species as SB-2 and introduced by SB-2s own edit. src/warp.rs 53-55 now reads The alternatives (zero-fill, error) are recorded in the kernel-choice decision, DEC-024 (AC11) - DEC-024 records NEITHER; repo-wide grep for zero-fill returns exactly 3 hits, two unrelated (tests/ifd_reader.rs, tests/support/perturb.rs) and the third being this claim itself, and DEC-024s only out-of-extent text is one Consequences-Neutral line naming clamp-to-edge with nothing it was chosen over. Round 2 resolved this dangling DEC-* placeholder to a concrete id without reading DEC-024 to check the neighbouring claim - the exact check it DID perform for the develop.rs placeholder - converting an unverifiable pointer into a verifiably false citation with more authority, which is unrun-docs-carry-errors (AGENTS.md 16 rule 4) landing inside the correction pass itself. AC11 is also the wrong AC to tag: it governs the KERNEL choice (bilinear/bicubic/Lanczos and measured scores), not the out-of-extent rule. Same sentences which records no per-frame oracle scores is imprecise too - DEC-024 line 98 records -60.169 on L1021223.DNG - and the SB-3 fix should rewrite the whole sentence. NEW: FU-11, follow-up. cost.sessions round-2 build entry carries agent: claude-sonnet-5 while that sessions own handback documents claude-opus-5 across 97 metered messages and prices $34.26 at Opus rates; scripts/handback-sync.sh line 97 reads to_agent from the handoff, and HANDOFF-046s to_agent was left at round 1s claude-sonnet-5 when round 2 reused the file. Bad model attribution reaches just calibration silently; one-field fix at ship, gates nothing, so FU not SB. Round-2 diff is otherwise clean: src/ changes are comment-only (0 non-comment changed lines, asserted), no existing test weakened (only 4 doc-header lines removed from tests/warp.rs), DEC-024 byte-unchanged since 40f5d45, SPEC-018.md body and ACs untouched, decisions-audit 0 structural errors, lint-ci green on pinned clippy 0.1.98, fmt clean, DEC-024 confidence 0.85 so no 16 yellow flag. FU-1 through FU-10 all verifiably still live and still owed a ship disposition - FU-5s truncation is still visible as notes: 12/14 ACs green; AC8/AC9 in the round-1 build session. cost.sessions structure is otherwise coherent: 3 entries totalling 153,410,303 which sums correctly. PR not opened; nothing repaired."
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 5579812
      estimated_usd: 15.02
      duration_minutes: 14
      recorded_at: 2026-09-07
      notes: "Round 3 (punch-list) on HANDOFF-048's one ship blocker, SB-3 CLOSED at b92b30c. One file, one rustdoc paragraph - src/warp.rs 53-55 replaced by 53-62, 10 insertions and 3 deletions, all inside doc comments, so decoded pixel output cannot move and the suite stays at 206 passed / 0 failed / 2 ignored. SB-3 was three defects in one sentence and all three are fixed. (A) The invented alternatives are gone: DEC-024 records neither zero-fill nor error, and records no alternative at all for the out-of-extent rule - its Alternatives Considered, lines 138-168, is Options A, B and C, every one about the kernel or the DEC-005 threshold. Re-measured rather than copied: grep -rn zero-fill over the tree returns 9 hits, 6 in process documents that discuss this very finding and 3 in source (tests/ifd_reader.rs 333, tests/support/perturb.rs 14, and the claim itself); grep -c on DEC-024 returns 0. (B) AC11 replaced by AC7: AC11 governs the KERNEL choice with measured per-frame scores (SPEC-018 428-433), while AC7 is the criterion that pre-registered the out-of-extent rule (SPEC-018 387-395) and is the AC DEC-024 itself names in the same clause that records the choice. (C) The 'records no per-frame oracle scores' clause is replaced by the measurement DEC-024 does hold - AC8 at -60.169 on L1021223.DNG (DEC-024 97-98) - reproduced by running the reader rather than quoted from the handback, since just test prints that same figure back in the ignore reason for warp_scores_at_least_eightyfive_via_spec_020_oracle. Phrased non-exclusively because DEC-024 also records AC9's -60.193 / -55.075 pair; a cleft would have been a fresh 16-rule-1 over-generalisation inside the sentence fixing an imprecision. WARNING, the one judgement call: the dispatch's suggested wording, clamps to edge per DNG 6.4.1, would have inherited SB-3's own precondition. DEC-024 195-196 records that 6.4.1 is SILENT on the out-of-extent rule, confirmed by full-text search, and src/warp.rs 36-43 - the reverify-approved heading five lines above the rewrite - says the same, so citing 6.4.1 as the SOURCE of a clamp rule would have been the same species of false citation one paragraph later. The rewrite takes the intent (the honest citation is the DNG spec and AC7, not a DEC that never weighed it) and states it as 6.4.1's SILENCE plus AC7, which is what both sources support. DEC-024 is byte-unchanged: git diff 08ad42e..b92b30c -- decisions/ is empty. Gates on b92b30c: lint-ci on the PINNED clippy 0.1.98 (not local 0.1.97), cargo fmt --check clean - note just fmt is NOT a recipe in this repo, fmt runs inside just lint - just test at 206 passed / 0 failed / 2 ignored run TWICE, once with IRRADIANCE_CORPUS_DIR pointed at the real corpus (4 frames present) and once with it unset (12 SKIP lines, default root absent), identical counts both ways; just deny licenses ok; just msrv 1.90.0; just fuzz-warp 23,438,651 runs in 61 s, zero crashes, cov 172 ft 457. CI GREEN on b92b30c - push run 34173566347 and pull_request run 34173569505, 10 jobs each, all success, fuzz smoke warp_opcode included, both watched to completion with gh run watch --exit-status rather than inferred. Nothing else touched: FU-1 through FU-11 all still open and owed a ship disposition, DEC-024, SPEC-018's body and ACs, src/lib.rs's SB-2 rewrite, src/develop.rs, every test including SB-1's, and PR 16, which was open on arrival and received a git push only. WARNING for the orchestrator: FU-11 will bite this entry too. HANDOFF-046's to_agent is still claude-sonnet-5 from round 1 and handback-sync line 97 reads it, but all 51 metered messages this session report claude-opus-5, so the round-3 cost.sessions entry will carry the WRONG agent exactly as round 2's did. Left unedited because the dispatch forbids the fix; FU-11 restated with one more instance."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 3275009
      estimated_usd: 11.64
      duration_minutes: 71
      recorded_at: 2026-09-07
      notes: "APPROVED on b92b30c - SB-3 CLOSED, no new SB, round 3's diff is comment-only. Reconciled against git and disk rather than the handback (DEC-004 rule 1): git diff 32599d2..b92b30c is src/warp.rs ALONE, 13 changed lines, 0 non-comment (asserted, not eyeballed), so no decoded pixel can move and SB-1's test, SB-2's src/lib.rs rewrite, src/develop.rs, tests/ and decisions/ are all byte-untouched this round. The three SB-3 defects are each fixed and each verified against the SOURCE document, not against the handback. (A) The invented alternatives are gone: grep -rn zero-fill src/ returns 0 hits (asserted), the only surviving source hits are tests/ifd_reader.rs 333 and tests/support/perturb.rs 14, both unrelated, and DEC-024 contains the string zero-fill 0 times. I asserted the ABSENCE claim itself rather than trusting it, since an absence is the easiest thing to assert falsely: 'alternativ' appears in DEC-024 EXACTLY ONCE, at line 138, the Alternatives Considered heading, and 'out-of-extent' appears EXACTLY ONCE, at line 195, so 'records the choice in a single Consequences Neutral line' and 'It records no alternative against it' are both literally true rather than merely plausible. Alternatives Considered spans 138-168 and holds exactly Options A (bicubic or Lanczos-3), B (relax DEC-005's 85 threshold) and C (chosen, keep bilinear plus the analytic checks), so 'entirely kernel and oracle threshold' is accurate. (B) AC11 is gone from src/ entirely - grep -rn AC11 src/ returns 0 hits - and AC7 is the right criterion: SPEC-018 at b92b30c lines 387-395 pre-registers the out-of-extent rule using AC7's own word 'pre-registered', and DEC-024 195-202 names AC7 (outside_pixels_follow_the_dng_spec_rule) as what pins the choice, on simplicity and determinism grounds, so the rustdoc matches the record clause for clause. The build's SPEC-018 line citations read 8 low against HEAD only because f9c43ee later inserted the round-3 cost entry above them; at the SHA the build actually read they are exact. (C) I RE-MEASURED AC8 instead of reading it back: cargo test --test warp -- --ignored --exact warp_scores_at_least_eightyfive_via_spec_020_oracle printed 'AC8 LEICA-Q2-MONO/L1021223.DNG: SSIMULACRA2 score = -60.169' in 135.92 s against dnglab 0.7.2, matching DEC-024 97-98 exactly. Worth recording that the round-3 handback's own verification story for this number is weaker than it reads - 'just test prints it back in the ignore reason' prints a STATIC string in tests/warp.rs, not a fresh measurement - but the number is right, I ran the reader, and the shipped sentence cites DEC-024, which does hold it. THE JUDGMENT CALL IS CORRECT AND I WOULD HAVE MADE IT. The dispatch's suggested 'clamps to edge per DNG 6.4.1' would have asserted a rule that DEC-024 195-196 records the specification does NOT hold, one screen below src/warp.rs line 36's own reverify-approved heading 'DNG 1.7.0.0 is silent; this build clamps' - unrun-docs-carry-errors landing inside the pass that exists to fix unrun-docs-carry-errors, which is exactly the instance-4 shape signals.yaml calls the one that settles it. The rewrite rests the rule on 6.4.1's SILENCE plus AC7 and on nothing else, and it did so without opening a specification it could not verify: it leaned on the two in-repo documents it did read. src/warp.rs cites 6.4.1 seven times and not once as the source of a clamp rule. The closing clause, 'those measurements are what established that the oracle cannot validate this stage at all', is honest rather than over-general: DEC-024's Finding 2 puts the measurements first and labels the dnglab/rawler source inspection at 116-130 as the ROOT CAUSE, and the rustdoc defers the why to the Kernel section below, which states it. No clause of the ten new lines over-generalises, so no section 16 rule 1 finding either. Gates re-run by me on b92b30c: just lint-ci green on the PINNED clippy 0.1.98 while this machine's default is 0.1.97; just test 206 passed / 0 failed / 2 ignored run TWICE, once with IRRADIANCE_CORPUS_DIR at the real corpus (7/7 present) and once with it UNSET (7 SKIP lines, default root absent), identical counts both ways, so the tier-B discipline holds; decisions-audit clean; DEC-024 confidence 0.85, so no section 16 yellow flag. CI GREEN on b92b30c confirmed by me from the API rather than inferred from a green predecessor: push run 34173566347 and pull_request run 34173569505, headSha b92b30cea0db66eeb0d2324fe7816d2365bc362f on both, conclusion success, 10 jobs each all success including fuzz smoke warp_opcode. cost.sessions holds 5 entries summing to 165,780,803, matching totals exactly, and 168.25 USD likewise; the round-3 build entry does carry agent claude-sonnet-5 while that session ran opus-5, which is FU-11 landing a second time exactly as HANDOFF-049 predicted, NOT a new finding. FU-1 through FU-11 are all still open and none was re-opened or silently closed by this round; FU-5's truncation is still visible verbatim at SPEC-018 line 111 as an unquoted notes value ending at 'AC8/AC9'. ONE OBSERVATION for ship, deliberately unnumbered because it predates round 3 and sits outside this handoff's scope: AGENTS.md line 1394 still states unrun-docs-carry-errors at N=5 while guidance/signals.yaml line 76 records N=6, instance 6 dated 2026-09-06 from SPEC-020 verify - the rule's own instance count is stale in the file agents read first, and this dispatch cited N=6. Cost method: 37 metered messages deduped by message.id from my OWN transcript, identified by the scratchpad UUID 431a772f-caec-4323-b122-b8231db1ab28 rather than by text-matching, all reporting message.model claude-opus-5, so tier_map.verify's prediction and this handoff's to_agent were both right for the third round running. Duration is WALL CLOCK 71 minutes, of which a single 57-minute idle gap awaited the user's continue, so active work was 14 minutes - recorded unhidden so calibration can discount it. PR not opened, handback-sync not run, cost.sessions not hand-edited, nothing repaired."
    - cycle: ship
      agent: claude-opus-4-7
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-07
      notes: "main-loop, not separately metered — orchestrator's ship pass writes the Follow-ups table with 12 dispositions (SB-1..3 fixed; FU-1, FU-2 fixed inline via doc-only edits to DEC-024, src/develop.rs, SPEC-018 Implementation Context, and AGENTS.md; FU-3, FU-4, FU-6, FU-7 closed with reasons; FU-5, FU-9, FU-11 filed as new signals; FU-8 closed as spec-sanctioned name; FU-10 filed as SPEC-021 frame). Non-null enforcement (AGENTS.md section 4) exempts design/ship."
  totals:
    tokens_total: 169055812
    estimated_usd: 179.89
    session_count: 7
---

# SPEC-018: WarpRectilinear radial geometric correction

## Context

`SPIKE-001` parsed `OpcodeList3` out of **one Q2M file** (`L1021223.DNG`)
straight from bytes and measured **~504 px of inward displacement at the
corner — 6 % of image width**. Coefficients on **that one frame**
(`kr0..kr3`, optical centre `(0.5, 0.5)`, no tangential terms):

```
kr0  0.999251106            kt0 0.0            f(1)  = 0.899841
kr1 −0.06137651287726358    kt1 0.0            f(0.75) = 0.944957 → −207.7 px
kr2 −0.09391554139336016                       f(0.50) = 0.978910 →  −53.0 px
kr3  0.05588200921529175                       f(0.00) = 0.999251 →   −0.0 px
```

⚠ **The coefficients are per-frame, not a camera constant** — `SPEC-020`
verify (`SPEC-020/FU-1`, 2026-09-06) parsed OpcodeList3 out of all three
decodable Q2M frames and measured:

```
                kr0             kr1              kr2              kr3
L1021223  0.9992511060  −0.0613765129  −0.0939155414  0.0558820092
L1026016  0.9992511060  −0.0418451693  −0.1023114788  0.0578767192
L1026192  0.9992511060  −0.0323314533  −0.1042041102  0.0563642935
```

Only `kr0` is constant. `kr1` varies **~1.9×** across the three frames. A
build that hardcodes L1021223's set as *the* Q2M coefficients will be
wrong on the two other frames — the same failure mode `unrun-docs-carry-
errors` instance 2 hit on `Orientation` for L1026016. **`AC1` reads
`kr0..kr3` from each file's own OpcodeList3, and AC4/AC8's expected
displacements are per-frame.** `SPIKE-001`'s single-frame measurement
stays as evidence but is no longer the source of the tier-A hex fixture:
each frame's own hex fixture ships in `tests/oracle-fixtures/`.

`STAGE-003` records this as **NOT DEFERRABLE**: skipping the warp does not
produce a slightly-off image, it produces one that is visibly wrong by 6 %
at the corners, and no dnglab reference render would ever match. Consistent
with the Q-series 28 mm lens being designed around software correction. This
spec is the single item in `PROJ-001` that most directly decides whether the
project's thesis holds.

⚠ **Corrected at build (`DEC-024` Finding 1):** this paragraph originally
said `OpcodeList3` runs after cropping. Re-reading DNG 1.7's own
`DefaultCropOrigin`/`DefaultCropSize` text (they describe "the origin/size
of the **final** image area... relative to `ActiveArea`") found that
backwards: `OpcodeList3` runs over the normalised **`ActiveArea`** window,
and `DefaultCrop` extracts the smaller "final" rectangle **after** it, not
before. For Q2M this changes the warp's own coordinate extent by under 1%
(`ActiveArea` 8392x5632 vs `DefaultCrop` 8368x5584); a future camera with a
larger crop margin could differ materially. Its geometric correctness was
meant to be the SPEC-020 oracle's headline case — DEC-005's falsifier is
literally **"a missing warp must land far below 85"**, measured −68 — but
`DEC-024` Finding 2 found that oracle cannot see this feature at all
(`dnglab`/`rawler` do not implement DNG `OpcodeList` processing). Building
this spec **without** SPEC-020 in place is the trap `SPEC-015` existed to
break, applied to STAGE-003; hence `depends_on: [SPEC-020]` — the
dependency was still the right call, even though the oracle it unlocked
could not do the job assumed here.

## Goal

Parse `OpcodeList3` from the file, apply `WarpRectilinear` per DNG spec
1.7.0.0 § 6.4.1 to the normalised+cropped image, and land in a state where
the developed Q2M frames score **≥ 85** against `dnglab analyze --srgb`
through SPEC-020's oracle — on a decoder that never panics on an
attacker-controlled opcode stream, and with the resampling kernel choice
recorded in a `DEC-*` rather than smuggled in as an implementation
accident.

## The design decision this spec rests on

⚠ **The resampling kernel is a decision, not a default.** DNG § 6.4.1
specifies the *transform*, not the *kernel that samples it*. Bilinear is
cheapest and reproducibly deterministic; bicubic is closer to what real
raw converters emit; Lanczos is highest quality and the slowest. The
SPEC-020 oracle scores `≥ 85` against dnglab, so the kernel must produce
output close **enough** to dnglab's kernel for the oracle to pass — but
the goal is not "match dnglab bit-for-bit" (`docs/oracle-contract.md`
§ "This oracle is single-sourced": that would be matching rawler, not
being correct).

**Pre-registered rule for the kernel choice:** pick the simplest kernel
that lets the three decodable Q2M frames all score ≥ 85 through SPEC-020,
and record the alternatives considered in a `DEC-*`. If bilinear scores
≥ 85 on all three frames, ship bilinear — `DEC-002`'s determinism guidance
prefers the simpler operator. If it does not, upgrade one step and record
why in the DEC. Do NOT tune the score by parameter selection until the bar
is met; a kernel that needs tuning to pass is a wrong kernel.

**⚠ SPIKE-001's warp coefficients are measurements, not the spec's
authority.** The DNG-spec convention for `r`'s normalisation ("Assumes the
DNG convention that r normalizes to 1.0 at the corner. The pixel figures
should be confirmed against the spec before being quoted as exact" —
`SPIKE-001` § "Q4") must be resolved against DNG 1.7.0.0 during the
design-time probe below, not carried forward from the spike. If the spec
says half-width normalisation, the corner displacement figure changes and
so does every expected test value.

## Inputs

- **`spikes/done/SPIKE-001-*.md`** — the measured coefficients, the caveat
  above, and the ~504 px empirical figure that anchors the corner test.
- **The DNG 1.7.0.0 specification, § 6.4.1 "WarpRectilinear"** — the
  authoritative source for: parameter order, radius normalisation
  convention, the `f(r) = kr0 + kr1·r² + kr2·r⁴ + kr3·r⁶` polynomial's
  domain, and the treatment of pixels whose source coordinate lands
  outside the input extent. **Read this before the parser is written**;
  every other input to this spec is subordinate to it.
- **`docs/oracle-contract.md`** — the develop-layer contract (SPEC-020),
  and § "This oracle is single-sourced" for the reason we don't chase
  bit-identity with dnglab.
- **`decisions/DEC-002-target-surface-parallelism-and-determinism.md`** —
  no `rayon` on the algorithmic path, output determinism pinned within a
  `develop_version`. The warp resampler is on the algorithmic path.
- **`decisions/DEC-016-*.md`** — the caller-owned-buffer shape
  `develop_into` uses. `WarpRectilinear` writes into that same buffer;
  do NOT introduce a second allocator.
- **`decisions/DEC-018-*.md`** — the u16 full-scale representation with
  clamped levels. The warp reads and writes in that same representation.
- **`src/develop.rs`** — `develop_into` and `output_dimensions`. This
  spec's application point: after `Orientation`, before tone curve.
- **`src/ifd.rs`** — the `Sensor` type. `OpcodeList3` is an IFD tag
  (`0xC74E`); the parser is a new field on `Sensor` or a sibling reader
  called during develop, and this design pass chooses which (see
  `## Implementation Context`).
- **`fuzz/fuzz_targets/`** — the existing `ifd`, `plane`, `develop`
  targets; `warp_opcode` is a **new** target seeded from real
  `OpcodeList3` bytes (see `## Outputs`).
- **A real Q2M file** (`IRRADIANCE_CORPUS_DIR/L1021223.DNG` and the
  other decodable frames) — for the tier-B end-to-end score. `SPEC-015`'s
  three-frame set is the minimum.

## Outputs

- **`src/opcode.rs`** — new module. The `OpcodeList3` byte-stream parser:
  reads the length prefix, iterates opcodes, extracts the WarpRectilinear
  parameter block for `OpcodeID == 1`, skips optional-and-unknown opcodes
  (per DNG spec), errors on mandatory-and-unknown. Panic-free (typed
  errors, bounds-checked slice reads — `no-panics-on-untrusted-input`).
  A small `pub` surface: one entry point that returns `Option<WarpRect>`
  from opcode bytes, plus its error type.
- **`src/warp.rs`** — new module. The resampler: given `WarpRect` and
  an input plane in `develop_into`'s u16 representation, writes the
  warped output into a caller-owned buffer (same shape as `develop_into`).
  Kernel chosen per `## The design decision this spec rests on`.
- **`src/develop.rs`** — modified. `develop_into` gains a warp application
  step **after** `Orientation` and **before** the tone curve entry point
  (which is `SPEC-019`'s work; for this spec, the tone-curve step is a
  no-op — the warped u16 buffer is the output). `output_dimensions` is
  **unchanged** (the warp is inward, so the output extent is still the
  DefaultCropSize per DNG § 6.4.1).
- **`src/ifd.rs`** — modified. `Sensor` gains an `opcode_list_3:
  Option<Vec<u8>>` field, populated from IFD tag `0xC74E` at the
  same read path `SPEC-014`'s levels/geometry tags come through. The
  parse of these bytes into `Option<WarpRect>` lives in `src/opcode.rs`,
  not in `Sensor` — the "unread field" rule (AGENTS.md §11) is
  satisfied because `develop_into` calls the parser during develop.
- **`Cargo.toml` (root)** — no new dependencies. Bilinear/bicubic/Lanczos
  are all hand-implementable at the size this operation runs at (one
  plane per file, once per develop) and pulling `resize`/`image`
  crates violates `library-not-application`.
- **`fuzz/fuzz_targets/warp_opcode.rs`** — new fuzz target on the
  `OpcodeList3` byte-stream parser. Seeded from real Q2M files'
  `OpcodeList3` bytes plus 2–3 hand-truncated variants (matches
  `SPEC-003`'s seed shape).
- **`fuzz/Cargo.toml`** — one new `[[bin]]` entry for `warp_opcode`.
- **`app.just`** — one new recipe `fuzz-warp` following the shape of
  `fuzz-ifd` / `fuzz-plane` / `fuzz-develop` (the `+toolchain` trap
  applies here too — `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo
  +nightly fuzz run warp_opcode ...`), and one line in AGENTS.md §6's
  code block naming the new recipe (§6's rule 8: every recipe's commands
  appear in the block).
- **`tests/develop.rs`** and/or **`tests/warp.rs`** (new) — the failing
  tests below. State the choice; do not silently pick.
- **A `DEC-*`** for the resampling kernel choice, per `## The design
  decision this spec rests on`. Alternatives considered, why chosen,
  and the measured scores on the three decodable frames.
- **A `DEC-*`** for the `OpcodeList3` field shape on `Sensor` if the
  build finds a non-obvious choice — e.g. eager parse vs lazy parse
  during develop. Not required if the choice is "store the raw bytes on
  `Sensor`, parse inside `develop_into`", which is `develop`-side by
  construction and needs no decision record.
- **`docs/provenance-ledger.md`** — **two rows**: one for the
  `OpcodeList3` parser (class 1 — DNG 1.7.0.0 spec § 6.4.1), one for the
  WarpRectilinear application including the chosen kernel (class 1 for
  the transform, class 1 or 2 for the kernel depending on the source
  read for the resampling weights).

## Acceptance Criteria

- [ ] **AC1 — `OpcodeList3` parser reads real bytes.** Given the exact
      68-byte parameter block `SPIKE-001` measured on `L1021223.DNG`
      (`planes = 1`, `cx = 0.5, cy = 0.5`, `kr0..kr3` and `kt0..kt1 = 0.0`
      as listed in `## Context`), the parser returns a `WarpRect` whose
      coefficients round-trip to the same bytes when serialised. Byte-for-
      byte, with big-endian samples (DNG opcode-stream endianness).
      **Test:** `opcode_list_3_round_trips_the_q2m_warp` (tier A, from
      a hex fixture committed at `tests/oracle-fixtures/opcodelist3-
      L1021223.hex`).
- [ ] **AC2 — parser is panic-free on adversarial input
      (`no-panics-on-untrusted-input`).** No `unwrap` / `expect` / `panic`
      / indexing-that-can-fault on any parser path. Verified by (i) clippy
      lint policy already forbids it on the library (SPEC-006), and
      (ii) the fuzz target `warp_opcode` runs 60 s in CI-adjacent
      conditions with no crashes (**local** 60 s during design/build; the
      CI job is a smoke run per §12 bar 2). Seed corpus: the real bytes
      from AC1 plus at least three hand-truncated variants (`OpcodeList3`
      with the length prefix lying, with a zero-byte parameter block,
      with an unknown opcode id and the mandatory flag set).
      **Test:** `warp_opcode_fuzz_smoke_no_crashes_after_60s` (tier A,
      calls `cargo fuzz` behind `#[test]` guarded by a feature — or,
      simpler, runs the fuzz seeds through the parser directly and
      asserts no panic on any of them).
- [ ] **AC3 — parser skips optional-and-unknown opcodes, errors on
      mandatory-and-unknown (DNG § 6.4).** Two hand-built streams
      exercise the two paths.
      **Tests:** `opcode_parser_skips_optional_unknown` and
      `opcode_parser_rejects_mandatory_unknown` (tier A).
- [ ] **AC4 — WarpRectilinear application produces the SPIKE-001 corner
      displacement, within resampling tolerance.** On a **synthetic**
      input of the Q2M crop size (8368×5584) with a known impulse pattern
      at the corner (see `## Implementation Context` for the fixture),
      the warped output shows a peak at the source coordinate the
      polynomial predicts — within ± 1 pixel horizontally and vertically
      (the kernel's own resampling uncertainty). SPIKE-001's ~504 px
      corner displacement is the anchor.
      **Test:** `warp_moves_corner_impulse_to_spike_001s_source_coord`
      (tier A, synthetic; no corpus needed).
- [ ] **AC5 — WarpRectilinear is a NO-OP when coefficients would produce
      identity.** For `kr0 = 1.0, kr1 = kr2 = kr3 = 0.0, kt0 = kt1 = 0.0`,
      every output pixel equals the corresponding input pixel exactly
      (bit-equal in u16). The identity path is where a resampler most
      commonly hides a defect (a stride error that cancels itself on a
      symmetric input); the identity assertion catches it.
      **Test:** `identity_warp_is_the_identity_transform` (tier A).
- [ ] **AC6 — output extent equals `DefaultCropSize`.** Per DNG § 6.4.1
      the warp maps the input extent to itself; the shipped output
      dimensions are unchanged from `SPEC-014`'s `output_dimensions`.
      `git diff` on `output_dimensions` is empty. Tested implicitly by
      `AC7` (the score would fail on a size mismatch) and explicitly
      by an assertion that `output_dimensions` matches SPEC-014's
      documented values on the decodable frames.
      **Test:** `output_dimensions_are_unchanged_by_warp` (tier A).
- [ ] **AC7 — pixels whose source lands OUTSIDE the input extent are
      handled per the DNG spec.** DNG § 6.4.1's convention (clamp / zero /
      undefined) is read from the spec during design and pre-registered
      here — do NOT choose based on what scores best. If the spec says
      clamp-to-edge, clamp. If it says undefined, pick one and record in
      the kernel-choice DEC. The test asserts pixel values in the
      outer-corner ring the polynomial cannot reach match the chosen
      rule.
      **Test:** `outside_pixels_follow_the_dng_spec_rule` (tier A).
- [ ] **AC8 — `develop_into` output on all three decodable Q2M frames
      scores ≥ 85 through SPEC-020's oracle.** This is the primary
      acceptance — the SPIKE-001 falsifier ("a missing warp must land
      far below 85") is what the oracle is calibrated on, and it
      **must land at or above 85** on a correct warp. Report the
      **measured score per frame** in the handback. If any frame scores
      below 85, that is a **finding, not a threshold to relax** — same
      rule as SPEC-015/AC1.
      **Test:** `warp_scores_at_least_eightyfive_via_spec_020_oracle`
      (tier B — the corpus is required; skip loudly when absent per
      SPEC-002).
- [ ] **AC9 — the red-proof: mutating a coefficient turns the score
      below 85 (`oracle-must-be-shown-red`).** Applied via mutation of a
      TEST COPY (SPEC-013's precedent, DEC-017's mechanism): replace
      `kr1` with `0.0` in the parser output on one frame and rerun the
      score; the result MUST drop below 85. Actual scores at design
      unknown, but SPIKE-001's calibration puts the missing-warp full-
      subtraction at −68 — a partial subtraction (one term of four) can
      reasonably be expected to land ≤ 70 at ¼ res. Pre-registered rule:
      **the mutated score must be < 85** and the delta from the honest
      score must be recorded in the handback. Tier B (needs the real
      frame); the mutation is CODE-side, not opcode-bytes-side.
      **Test:** `warp_oracle_is_red_on_a_zeroed_kr1_coefficient` (tier B).
- [ ] **AC10 — the tier-A red-proof: mutating a coefficient turns
      `AC4`'s displacement test red WITHOUT the corpus** (SPEC-015/FU-10's
      lesson, made an AC). `AC9` needs the corpus and CI runs 0/7 corpus
      files; `AC4` runs on synthetic input, and its mutated form (`kr1 =
      0.0`) must produce a **different peak location** than the honest
      polynomial by at least 20 pixels on the synthetic-corner-impulse
      test. Runs with `IRRADIANCE_CORPUS_DIR` unset.
      **Test:** `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more`
      (tier A).
- [ ] **AC11 — the WarpRectilinear kernel choice is a `DEC-*`, with
      measured scores.** DEC states: alternatives considered (bilinear,
      bicubic, Lanczos), the chosen kernel, the measured score on each
      of the three decodable frames, and — if not bilinear — the
      measured shortfall bilinear had. The pre-registered rule (see
      `## The design decision this spec rests on`) is enforced by the
      DEC's content.
- [ ] **AC12 — no `rayon`, no runtime SIMD dispatch, output determinism
      pinned within `develop_version` (DEC-002).** The warp resampler
      runs single-threaded, with a compile-time-fixed kernel. Verified
      by (i) `Cargo.toml` has no `rayon`, (ii) the code contains no
      `is_x86_feature_detected` / runtime feature-dispatch call, and
      (iii) running the develop pipeline twice on the same input
      produces bit-identical output.
      **Test:** `warp_output_is_bit_identical_across_two_runs` (tier A).
- [ ] **AC13 — provenance rows for both new algorithms.**
      `docs/provenance-ledger.md` gains one row per module:
      `src/opcode.rs` (DNG 1.7.0.0 spec § 6.4.1, class 1) and
      `src/warp.rs` (transform: DNG 1.7.0.0 § 6.4.1, class 1; kernel:
      class 1 if implemented from a published paper or class 2 if from
      a permissive-crate reading). Honesty per §16 rule 3.
- [ ] **AC14 — eleven gates + `just lint-ci` + `just fuzz-warp`, CI
      observed green on the shipping SHA.** The `fuzz-warp` recipe is
      new; its 60 s smoke run must be added to CI in the same PR (per
      §12 bar 2: fuzz targets arrive with the parser, not retrofitted).

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum
across all.

- `opcode_list_3_round_trips_the_q2m_warp` — AC1, tier A
- `warp_opcode_fuzz_smoke_no_crashes_after_60s` — AC2, tier A (may run
  as an integration `#[test]` that shells to `cargo fuzz` — state the
  choice in the handback)
- `opcode_parser_skips_optional_unknown` — AC3, tier A
- `opcode_parser_rejects_mandatory_unknown` — AC3, tier A
- `warp_moves_corner_impulse_to_spike_001s_source_coord` — AC4, tier A
- `identity_warp_is_the_identity_transform` — AC5, tier A
- `output_dimensions_are_unchanged_by_warp` — AC6, tier A
- `outside_pixels_follow_the_dng_spec_rule` — AC7, tier A
- `warp_scores_at_least_eightyfive_via_spec_020_oracle` — AC8, tier B
- `warp_oracle_is_red_on_a_zeroed_kr1_coefficient` — AC9, tier B
- `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more` — AC10,
  tier A
- `warp_output_is_bit_identical_across_two_runs` — AC12, tier A

## Non-Goals

- **Any opcode other than WarpRectilinear.** `OpcodeList1`'s
  `FixBadPixelsConstant` is `SPEC-017`; `OpcodeList2` is empty on Q2M
  and not addressed here. Unknown opcodes are skipped/rejected per the
  DNG spec (AC3) — this spec does not implement them.
- **A general-purpose image resampler.** The kernel exists to serve
  `WarpRectilinear` on one plane at Q2M resolution. If SPEC-019 or a
  later spec needs resampling, it can share this module or grow its own;
  pre-generalising to N kernels or arbitrary-sized planes is not asked
  for here.
- **Tangential terms (`kt0`, `kt1`).** They are present in the DNG spec
  and the parser reads them (AC1), but every Q2M file measured is
  `kt0 = kt1 = 0.0`, so the applier can implement the pure-radial path
  and treat non-zero tangentials as a stop-and-report finding rather
  than a codepath. Note this in the module: a colour camera in PROJ-002
  may need tangentials, and that camera's spec adds them then.
- **Colour lens-shading (`OpcodeList3`'s `GainMap`).** Monochrome does
  not carry gain maps; PROJ-002 territory.
- **Bit-identity with dnglab's warp output.** Rejected in
  `docs/oracle-contract.md` § "This oracle is single-sourced": that
  target is matching rawler, not being correct. The oracle bar is
  perceptual (≥ 85), and the kernel choice DEC says why.
- **A crustyimg-side integration probe.** SPEC-020 handles the metric
  wiring; SPEC-018's job is the render. crustyimg's integration point
  is `STAGE-004`.
- **Recalibrating DEC-005's ≥ 85 threshold at full resolution.**
  DEC-005 marks this as a revisit condition; it happens **when**
  SPEC-018 lands, but it's not one of this spec's acceptance criteria —
  it's a follow-up if AC8 lands materially different from the ¼-res
  calibration.

## Notes for the Implementer

- **DNG § 6.4.1 first.** Every other input is subordinate. Read the
  spec's parameter order, radius normalisation, and out-of-extent rule
  before writing the parser or the applier. `SPIKE-001`'s
  parenthetical caveat about `r` normalisation is why.
- **The opcode stream is BIG-ENDIAN**, unlike the TIFF payload the
  reader already handles. That is easy to get wrong under a `SPEC-003`
  reader whose fluency is little-endian. Write one round-trip test
  first, on the SPIKE-001 hex fixture, and treat any parser change
  that alters its output as a defect.
- **Panic-free applies to the whole parser, not just the entry
  point.** Slice reads use `.get(range).ok_or(err)?`; `usize`
  conversions use `.try_into().map_err(err)?`; multiplication that
  could overflow uses `checked_mul` (parameter counts CAN be
  attacker-controlled). SPEC-003's `ifd.rs` is the precedent.
- **The resampler samples at `f64` and quantises to `u16` at the
  edge.** `DEC-018` is the analogue: the u16 boundary is where
  rounding is a decision. If bilinear is chosen, round-to-nearest
  (`(x + 0.5).floor() as u16` with clamp to `[0, 65535]`) matches
  `DEC-018`'s convention. If bicubic or Lanczos, negative overshoot
  can occur — clamp, do not truncate `u16` semantics.
- **The tier-B corpus tests skip loudly when the corpus is absent.**
  AC8's score assertion is on the real frames; AC10 exists because CI
  runs 0/7 corpus files and the tier-A red-proof is the only proof CI
  can see.
- **`just lint-ci`, not `just lint`.** Homebrew's clippy is 0.1.97; CI
  is 0.1.98 — the divergence has cost this repo 17 consecutive red
  runs.
- **The fuzz target's PATH= prefix is the `+toolchain` trap in a new
  suit.** `~/.cargo/bin/cargo +nightly fuzz run warp_opcode …` alone
  fails (the INNER `cargo build` inside `cargo fuzz` still resolves to
  Homebrew's stable). Match the shape of `just fuzz-ifd` in `app.just`.
- **If AC8 fails with the chosen kernel, do NOT change AC8 — change
  the kernel** (per `## The design decision this spec rests on`).
- **If any Q2M frame's warp coefficients differ from SPIKE-001's, that
  is a finding worth reporting.** The parser is right if the frames
  round-trip byte-for-byte; a coefficient set that would render
  meaningfully differently across the corpus is a real signal.

## Implementation Context

> This section carries the design-time probes required before build
> (AGENTS.md §12 "Design-time probe / measure-before-build"). The
> SPIKE-001 measurements are here as evidence with provenance; the DNG
> spec resolution is a **required probe** the build cycle must complete
> before it writes the parser.

### Confirmed in design

- **The warp coefficients** are as SPIKE-001 measured them on
  `L1021223.DNG` and `L1026016.DNG`. Radial-only, four terms, optical
  centre `(0.5, 0.5)`. Reproducible by rereading `OpcodeList3` at
  IFD tag `0xC74E`, byte offsets per the DNG opcode format.
- **The failure mode of skipping the warp**: ≈504 px inward at the
  corner (~6 % image width) on `SPIKE-001`'s frame. SSIMULACRA2 score
  −68 against dnglab's `--srgb` at ¼ resolution (`DEC-005`
  calibration).
- **`OpcodeList3` runs on the `ActiveArea` image, BEFORE `DefaultCrop`
  extraction and `Orientation`** in the monochrome pipeline (colour
  transforms are absent). Corrected at build (`DEC-024` Finding 1); the
  earlier "after cropping and orientation" claim was `SPEC-018`'s own
  design-time assumption, not a spec statement.
  The output extent equals the DefaultCropSize — the warp is a
  spatial rearrangement of the same-sized image.
- **`no image crate`**: `library-not-application` is standing; the
  resampler is hand-written. Bilinear is ~20 lines including the
  clamp; bicubic is ~40; Lanczos-3 is ~60. Any of the three fits
  within the panic-free discipline.

### Required before build handoff

- **Resolve `r`'s normalisation against DNG 1.7.0.0 § 6.4.1.**
  SPIKE-001's caveat: **"Assumes the DNG convention that r normalizes
  to 1.0 at the corner. The pixel figures should be confirmed against
  the spec before being quoted as exact."** The build must read the
  spec and cite the exact clause in the DEC-*. If the convention is
  half-width, the corner displacement figure and every AC4 expected
  value derive from a different `r_max`.
- **Resolve the out-of-extent pixel rule against DNG 1.7.0.0 § 6.4.1.**
  Clamp / zero / undefined / spec-silent. Cite the clause; if
  spec-silent, choose and record in the kernel-choice DEC.
- **Confirm the WarpRectilinear opcode ID and version at the byte
  level from a fresh IFD parse** — SPIKE-001 said `OpcodeID == 1,
  version 1.4.0.0`, but re-parse to be sure, and record the exact
  bytes in the tier-A hex fixture.

### Required in build (pre-registered)

- **Kernel selection.** Bilinear first. Measure the SPEC-020 score on
  all three decodable frames. If all ≥ 85, ship bilinear. If not,
  bicubic next, remeasure. If bicubic doesn't reach it either,
  Lanczos-3 as the last option. Record the measured scores per kernel
  in the DEC — the alternatives-considered section is data, not prose.
- **Corner displacement fixture.** Design a synthetic input that
  makes AC4 diagnosable: a single-pixel impulse (or small Gaussian) at
  the corner of a large flat field, so the resampled peak's location
  is the whole content. The fixture is committed under
  `tests/oracle-fixtures/`; state its byte size in the handback (≤
  100 KB target).
- **Fuzz seed set for `warp_opcode`.** Real OpcodeList3 bytes from
  at least two Q2M frames, plus three hand-written truncations. Match
  `SPEC-003`'s seed shape; `just fuzz-seeds` regenerates the seeds
  from `examples/fuzz-seeds.rs`.
- **Measured full-develop wall-clock** (info only, reported in
  handback). If it exceeds ~10 s per frame in release, note it — the
  crustyimg consumer will notice.

### Not measured, and why

- **Full-resolution recalibration of the ≥ 85 threshold**: still a
  `DEC-005` follow-up, but this spec's landing produces the first
  real developed output. If AC8 lands materially different from the
  ¼-res calibration, that observation writes the recalibration; it is
  a `SPEC-018`-cycle follow-up, not an AC.
- **Perceptual score against a *second* independent reference render.**
  There is no second permissively-runnable reference for Q2M. PROJ-002's
  ColorChecker ΔE is the true independent check, and it needs a
  colour camera.

## Follow-ups

| id | finding | disposition |
|---|---|---|
| `SB-1` | `develop_into`'s warp branch had no live test — reverting to crop-then-warp compiled, changed output, kept the suite at 205/0/2. | `fixed` — round 2 added `develop_into_crops_from_the_warped_active_area_not_the_warped_crop` (tests/warp.rs); red-proof observed both directions with mutation md5 change; round 3 kept the fix intact. Ship SHA `b92b30c`. |
| `SB-2` | Two false rustdoc claims: `src/warp.rs:59-61` (fabricated per-frame oracle scores) and `src/lib.rs:56-57` (pre-Finding-1 pipeline order in the correcting commit). | `fixed` — round 2 rewrote both to match reality citing DEC-024; verify round 2 confirmed. |
| `SB-3` | Round-2's own SB-2 rewrite introduced a false DEC-024 citation in `src/warp.rs:53-55` — claimed DEC-024 (AC11) recorded "zero-fill, error" out-of-extent alternatives that DEC-024 records neither of. | `fixed` — round 3 rewrote lines 53-62 to cite § 6.4.1's silence + AC7 (not § 6.4.1 as the source of the clamp rule, per DEC-024:195-196), and named DEC-024's Consequences "Neutral" line + `-60.169 on L1021223.DNG` precisely. Verify round 3 approved (SHA `b92b30c`). |
| `FU-1` | DEC-024 Finding 1 and `src/develop.rs:460` cited `docs/measured-q2m-dng.md` as source of the wrong pipeline-order claim; `git log -S` shows that file has never contained the claim. | `fixed` — corrected at ship (commit `fd5cc1f`): both citations now attribute the wrong claim to SPEC-018's own design, not the corpus documentation. |
| `FU-2` | SPEC-018 `## Implementation Context` still carried two facts round 1 disproved: IFD tag `0xC740` (correct: `0xC74E`) and "runs after cropping and orientation" (correct per Finding 1). | `fixed` — corrected at ship (commit `fd5cc1f`): three `0xC740` → `0xC74E` and the pipeline-order line rewritten. |
| `FU-3` | DEC-024 never records DNG 1.7.0.0 p.104's recommendation to use "a suitable resampling kernel, such as a cubic spline"; Alternatives-Considered's kernel escalation was short-circuited when AC8 became unmeasurable and the spec's own recommendation was never weighed. | `closed` — bilinear ships on the pre-registered rule (measurement-driven escalation, no measurement possible), and AC4/AC10 are kernel-independent so no defect reaches a consumer. DEC-024 amendment recording DNG's recommendation is worth doing but is a doc-only follow-up any future DEC-024 revision can carry. |
| `FU-4` | DEC-024 does not cite DNG 1.7.0.0 p.104's radius-normalisation clause; `src/warp.rs`'s cross-check against SPIKE-001's ~504 px is circular (SPIKE-001 assumed the same convention). Verify verified the substance and it is correct — only the citation is missing. | `closed` — verify confirmed the substance is correct against DNG p.104; the missing citation is a doc-only follow-up and DEC-024's confidence 0.85 already flags this class of gap. |
| `FU-5` | `just handback-sync` silently truncated HANDOFF-046 round-1's notes at the bare `#` in `#[ignore]d`; the entire post-`#` finding dropped from `cost.sessions[build].notes` in the archived record. | `signal: handback-sync-treats-hash-as-comment-in-unquoted-notes` — new signal filed at bar 3, evidence points at HANDOFF-046. Sibling of the multi-line-scalar truncation signal; same script, same fix candidate. |
| `FU-6` | AC8 is unsatisfiable on L1026016.DNG for a SECOND independent reason DEC-024 does not record: `dnglab --srgb` ignores `Orientation` and emits 8368×5584 for an Orientation-6 frame while the correct render is 5584×8368; `develop_and_score`'s `assert_eq!` panics before scoring. | `closed` — evidence for SPEC-021's oracle-scope narrowing (FU-10). SPEC-021's Context will absorb this; recording it here as evidence rather than a live defect since AC8 is already `#[ignore]`d. |
| `FU-7` | AC4/AC10's fixture extent (8368×5584) is no longer the extent the shipped pipeline warps at (8392×5632). "503.7 px" describes the fixture; the shipped render corrects 506.1 px on L1021223 (439.7 / 408.9 on the other two, independently decoded). | `closed` — spec-literal (the fixture extent is what the AC pre-registers), verified geometrically correct against the shipped frame. Measurement recorded here as data; future doc updates can carry the 506.1 figure. |
| `FU-8` | `warp_opcode_fuzz_smoke_no_crashes_after_60s` runs no fuzzing and no 60 seconds — it pushes 6 seeds through the parser. Spec-sanctioned by AC2's Failing-Tests wording but the name is what a future reader greps. | `closed` — spec-sanctioned exact name from `## Failing Tests` (`named-tests-can-pass-vacuously` protection). Renaming both the test and the AC entry is a paired code + spec change, out of scope for ship; a future spec that touches the perceptual/warp fuzz layer can bundle. |
| `FU-9` | Finding 2's root-cause evidence came from reading LGPL `dnglab` source. DEC-024 discloses this as "feature-presence, not algorithm content" but `provenance-recorded-per-algorithm` says flatly "Reading a copyleft implementation is not permitted", and §15 says "its source is not a reference". Verify round 1 established the behavioural NCC-tile probe as sufficient. | `signal: dnglab-source-reading-tempts-behavior-check` — new signal filed at bar 3, evidence points at HANDOFF-046 and HANDOFF-047. Fix candidate is an AGENTS.md rule codifying behavioural probe as the sanctioned method for feature-presence questions about copyleft implementations. |
| `FU-10` | SPEC-020's develop-layer oracle is structurally broken for warp-bearing pixels (Finding 2). Ship SPEC-018 with `#[ignore]` on AC8/AC9 + a follow-up spec to formally narrow SPEC-020/DEC-005's scope, mirroring SPEC-015's analytic substitution. Fold in FU-6's orientation leg. DEC-005 confidence 0.80 materially weakened. | `spec: SPEC-021` — framed at ship (`projects/PROJ-001-monochrome-dng-develop/specs/SPEC-021-*.md`), STAGE-005 backlog, `depends_on: []`. Context absorbs FU-6 and FU-10; design happens JIT when picked up. |
| `FU-11` | `cost.sessions` round-2 AND round-3 build entries both carry `agent: claude-sonnet-5` but each session was actually claude-opus-5 (97 + 51 metered messages independently confirmed, priced at Opus rates in each handback). handback-sync reads HANDOFF-046's `to_agent` which was left at round-1's stale value across all three rounds. | `signal: handback-sync-inherits-stale-to-agent-across-punch-list-rounds` — new signal filed at bar 2 (N=2 same-session), evidence points at HANDOFF-046 rounds 2 and 3. Fix candidates in the signal notes; SPEC-018's calibration figures overstate Sonnet spend by ~19.9M Opus tokens (~$50) until repair. |

---

## Reflection

1. **What would I do differently next time?**
   — Run a scoring round-trip at design, not just byte parsing. Finding 2 (dnglab doesn't apply DNG opcodes) was discoverable in ~30 minutes of design-time work: score `dnglab --srgb` output against our own uncorrected develop pipeline; if scores are high, the reference isn't applying opcodes. Instead the discovery landed in the middle of round-1 build after 116M tokens of work and required a full SPEC-020 oracle re-scoping (SPEC-021). Also: coefficient-per-frame vs camera-constant would have been caught by reading OpcodeList3 out of all three decodable Q2M frames at design, not one — same pattern as `unrun-docs-carry-errors` N=6.

2. **Does any template, constraint, or decision need updating?**
   — Three signals filed this ship (`handback-sync-treats-hash-as-comment-in-unquoted-notes`, `dnglab-source-reading-tempts-behavior-check`, `handback-sync-inherits-stale-to-agent-across-punch-list-rounds`); each carries its own fix candidate. AGENTS.md §16 rule 4 evidence bumped N=5 → N=6 to match `signals.yaml` (stale for a day). DEC-005 (SPEC-020's ≥ 85 tolerance) is materially weakened by Finding 2 and gets its real revisit at SPEC-021's design. §12's fuzz-target-arrives-with-parser rule held cleanly. The template's rustdoc-vs-DEC-authority tension (SB-2, SB-3) is worth watching — a fix pass keeps writing false claims about the DEC being fixed to; a `just decisions-audit --claims` that greps rustdoc for `DEC-*` citations and checks each is a claim the DEC actually records could catch this class. That's a candidate for STAGE-005.

3. **Is there a follow-up spec I should write now before I forget?**
   — Yes: **SPEC-021** (framed at this ship) narrows SPEC-020/DEC-005's oracle scope to exclude warp-bearing pixels and folds in FU-6's Orientation-6 leg. STAGE-005 backlog.

4. **Where was the worst defect caught?** — `verify` (round 2 — SB-1's missing live test; a §15 check 8 escape class that would have shipped a warp branch with 0 live coverage into main). Finding 2 (round-1 verify) is a bigger structural finding but not a "defect caught" in the shipping-blocker sense — it's a real limitation of the reference implementation. SB-3 (round-2 verify) is the most instructive: a fix inheriting its own precondition, exactly the codified `a-fix-inherits` lesson landing inside the pass that names it.

5. **What can a user do now that they couldn't before?**
   — Before: `develop_into` produced an image visibly wrong by 6% at the corners on a Q-series Leica frame, because SPEC-018's WarpRectilinear radial correction was missing — no reference render would match. After: the develop pipeline unpacks the plane, normalises levels, applies OpcodeList3's WarpRectilinear over the ActiveArea (bilinear resampler, hand-written, no new dependency), then extracts DefaultCrop and applies Orientation. On L1021223.DNG the corner displacement is 503.7 px (matching SPIKE-001's measurement to sub-pixel accuracy), on L1026016 it's 437.7 px, on L1026192 it's 407.0 px — computed per frame from each frame's own coefficients. The oracle scope for warp correctness is now a known open question (SPEC-021) rather than an implicit assumption; AC4/AC10's analytic geometry check is the shipped gate.
