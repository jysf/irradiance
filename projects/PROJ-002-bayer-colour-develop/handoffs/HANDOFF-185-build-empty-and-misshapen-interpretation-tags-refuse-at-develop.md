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
  id: HANDOFF-185
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-23
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-089

project:
  id: PROJ-002
  stage: STAGE-029
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 14259009               # REAL combined count — what cost-audit reads
  estimated_usd: 11.23              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 18
  branch: feat/spec-089-misshapen-tags
  pr: https://github.com/jysf/irradiance/pull/319
  completed_at: 2026-11-23               # YYYY-MM-DD
  notes: "Floor, summed live from this session transcript (6c650547), 65 messages deduplicated by message.id, model claude-opus-5-5; in 131, cache creation 232,446 (1-hour), cache read 13,949,284, out 77,148. USD at Opus list rates 5 and 25 dollars per MTok with 1-hour cache writes. Taken before the handback edit and PR body, so re-sum after the session for the final figure. Duration is the transcript timestamp span."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-185: Empty and misshapen interpretation tags refuse at develop

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-089` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle.

## Context the Receiving Agent Needs

Read these, in this order, before writing code:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-089-empty-and-misshapen-interpretation-tags-refuse-at-develop.md`.
  The Acceptance Criteria, the Failing Tests, the rule-clause map and the distinguishing
  fixtures are the contract. Its Context has the design probe's reproduction and audit
  tables.
- **Decision:** `./decisions/DEC-078-a-malformed-interpretation-tag-the-develop-path-consumes-refuses-the-develop.md`,
  §7 (the amendment this spec implements), with §1 to §6 for the rule it extends.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-029-malformed-means-shape-too.md`.
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`.
- **`AGENTS.md`**: build, test and lint commands, coding and testing conventions.

### Relevant decisions

- `DEC-078` — §7 is this spec's rule; §2's sets, §3's order and `MalformedTag` are reused
  unchanged.
- `DEC-077` — its set refuses a malformed tag as `UnsupportedTag`; count zero now makes
  those tags malformed too.
- `DEC-076` — the colour code whose fallbacks §7.2 removes (`dual_illuminant`, the
  ForwardMatrix branch, `resolve_calibration`); its singular-matrix behaviour stands.
- `DEC-012` and `DEC-015` — the tolerant read and "absent is not recorded". Both hold.
- `DEC-051` — no new variant, so `tests/error_kind.rs` stays at 76. If you find you need
  one, stop and report it: that is a design change.

### Relevant constraints

- `no-panics-on-untrusted-input` — the new count and denominator checks read lengths and
  values without indexing.
- `oracle-must-be-shown-red` — AC9's mutants (a) to (j), each shown red then reverted,
  with a green control run first, on the distinguishing fixtures the spec names.

### Prior work

- `PR 312` / `HANDOFF-182` and `HANDOFF-183` — SPEC-087, the implementation this spec
  completes. `tests/malformed_tags.rs` is its test file; reuse its fixture, entry and
  sentinel machinery.
- The design probe (below). It reproduces both review findings and the audit table in
  the spec's Context, at c78ec4c. Start from it, turn its prints into assertions, and
  do not commit it as is.

## Expected Deliverables

- A PR against `main` from branch `feat/spec-089-misshapen-tags`.
- Write the failing tests first, see them red on `main`'s behaviour (that is the
  "fails on the original behaviour" evidence: record which tests went red), then
  implement.
- The Outputs list in the spec: `src/ifd.rs`, `src/color.rs`, the tests, and
  `CHANGELOG.md` `[Unreleased]` with the new refusals and the changed refusals of AC7.
- AC9's red-proof table in the handback: each mutant, the test that went red, and the
  green control.
- AC8's corpus re-measure in the handback: `irr ifd`'s `malformed_tags` on every held DNG,
  before and after.
- Gates before every push: `just test` (no extra arguments; it already passes
  `--all-features`) with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  so AC8's Tier-B pins run, `just lint-ci`, `just lint-no-allow`, `just fmt-check`,
  `just doc-check`, `just validate`.
- Report the env-gated SDK lanes (`IRRADIANCE_SDK_STAGE2_DIR`,
  `IRRADIANCE_SDK_COLOUR_DIR`) as SKIPPED when unset. Do not claim they ran.
- No re-pin and no `DEVELOP_VERSION` change. If a held file refuses or a Tier-B pin
  moves, stop and report it in the handback: the design's corpus measurement says
  neither happens.

### The design probe (throwaway, at c78ec4c)

Its output at c78ec4c: count-zero 274, 50714, 50717 and 50711 give
`malformed_tags=[]` and `Ok(())`; `ColorMatrix2` valid differs from absent, while 8
entries and zero denominators are byte-identical to absent. The audit test's output is
the second table in the spec's Context.

```rust
//! THROWAWAY probe (not committed): reproduce the two review findings at c78ec4c.
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use irradiance::color::develop_color_into;
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use tiff::Order;

const W: u32 = 8;
const H: u32 = 8;

fn desc(cfa: bool) -> DngDescription {
    let kind = if cfa {
        PlaneKind::Cfa { repeat_dim: [2, 2], pattern: vec![0, 1, 1, 2], plane_color: vec![0, 1, 2] }
    } else {
        PlaneKind::LinearRaw
    };
    let mut meta = DngMeta::default();
    if cfa {
        let m = |v: [[i32; 3]; 3]| -> Vec<(i32, i32)> {
            v.iter().flatten().map(|&n| (n, 10_000)).collect()
        };
        meta.color_matrix_1 = Some(m([[9000, -2000, -500], [-4000, 12000, 1500], [-500, 1500, 6000]]));
        meta.calibration_illuminant_1 = Some(17);
        meta.calibration_illuminant_2 = Some(21);
        meta.as_shot_neutral = Some(vec![(1, 2), (1, 1), (2, 3)]);
    }
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width: W,
        height: H,
        bits_per_sample: 16,
        kind,
        organisation: Organisation::Strips { rows_per_strip: H },
        payload: Payload::Samples((0..W * H).map(|i| 1000 + (i as u16) * 37).collect()),
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

fn load(d: &DngDescription) -> (Sensor, Vec<u16>) {
    let file = write_dng(d).unwrap();
    let c = Container::parse(&file).unwrap();
    let s = c.sensor().unwrap();
    let mut p = vec![0u16; (W * H) as usize];
    unpack_into(&s, c.byte_order(), &file, &mut p).unwrap();
    (s, p)
}

fn empty(ifd: TagIfd, tag: u16, ty: u16) -> RawTag {
    RawTag { ifd, tag, field_type: ty, count: 0, bytes: vec![] }
}

#[test]
fn finding1_count_zero() {
    for (tag, ifd, ty, cfa) in [
        (274u16, TagIfd::Ifd0, 3u16, false),
        (50714, TagIfd::Sensor, 4, false),
        (50717, TagIfd::Sensor, 4, false),
        (50711, TagIfd::Sensor, 3, true),
    ] {
        let mut d = desc(cfa);
        if tag == 274 {
            // Orientation is read from IFD0 or the sensor IFD; empty in both.
            d.extra_tags.push(empty(TagIfd::Sensor, tag, ty));
        }
        d.extra_tags.push(empty(ifd, tag, ty));
        let (s, p) = load(&d);
        let r = if cfa {
            let mut dst = vec![0u16; (W * H * 3) as usize];
            demosaic_into(&s, &p, &mut dst)
        } else {
            let (w, h) = output_dimensions(&s).unwrap();
            let mut dst = vec![0u16; (w * h) as usize];
            develop_into(&s, &p, &mut dst)
        };
        println!("tag {tag}: malformed_tags={:?} result={r:?}", s.malformed_tags);
    }
}

#[test]
fn finding2_colour_matrix_2() {
    let valid: Vec<(i32, i32)> = [[7000, -1500, -300], [-3500, 11000, 1800], [-300, 1200, 6500]]
        .iter()
        .flatten()
        .map(|&n| (n, 10_000))
        .collect();
    let cases: Vec<(&str, Option<Vec<(i32, i32)>>)> = vec![
        ("absent", None),
        ("valid", Some(valid.clone())),
        ("eight entries", Some(valid[..8].to_vec())),
        ("zero denominators", Some(valid.iter().map(|&(n, _)| (n, 0)).collect())),
    ];
    let mut out = Vec::new();
    for (name, cm2) in cases {
        let mut d = desc(true);
        d.meta.color_matrix_2 = cm2;
        let (s, p) = load(&d);
        let (w, h) = output_dimensions(&s).unwrap();
        let mut dst = vec![0u16; (w * h * 3) as usize];
        let r = develop_color_into(&s, &p, &mut dst);
        println!("CM2 {name}: malformed_tags={:?} result={r:?}", s.malformed_tags);
        out.push((name, dst));
    }
    println!("valid == absent: {}", out[1].1 == out[0].1);
    println!("eight == absent: {}", out[2].1 == out[0].1);
    println!("zero-den == absent: {}", out[3].1 == out[0].1);
}

fn rat(n: i32, d: i32) -> Vec<u8> { let mut v = n.to_le_bytes().to_vec(); v.extend(d.to_le_bytes()); v }
fn srats(vals: &[(i32, i32)]) -> Vec<u8> { vals.iter().flat_map(|&(n, d)| rat(n, d)).collect() }

#[test]
fn audit_related_readers() {
    let id9: Vec<(i32, i32)> = [1, 0, 0, 0, 1, 0, 0, 0, 1].iter().map(|&n| (n, 1)).collect();
    let cases: Vec<(&str, RawTag, bool)> = vec![
        ("ActiveArea count 3", RawTag { ifd: TagIfd::Sensor, tag: 50829, field_type: 4, count: 3, bytes: [0u32, 0, 8].iter().flat_map(|x| x.to_le_bytes()).collect() }, false),
        ("ActiveArea count 0", RawTag { ifd: TagIfd::Sensor, tag: 50829, field_type: 4, count: 0, bytes: vec![] }, false),
        ("DefaultCropOrigin count 1", RawTag { ifd: TagIfd::Sensor, tag: 50719, field_type: 4, count: 1, bytes: 1u32.to_le_bytes().to_vec() }, false),
        ("DefaultCropSize count 0", RawTag { ifd: TagIfd::Sensor, tag: 50720, field_type: 4, count: 0, bytes: vec![] }, false),
        ("BlackLevelRepeatDim count 0", RawTag { ifd: TagIfd::Sensor, tag: 50713, field_type: 3, count: 0, bytes: vec![] }, false),
        ("CFAPlaneColor count 0", RawTag { ifd: TagIfd::Sensor, tag: 50710, field_type: 1, count: 0, bytes: vec![] }, true),
        ("CalIllum2 count 0", RawTag { ifd: TagIfd::Ifd0, tag: 50779, field_type: 3, count: 0, bytes: vec![] }, true),
        ("CamCal2 8 entries", RawTag { ifd: TagIfd::Ifd0, tag: 50724, field_type: 10, count: 8, bytes: srats(&id9[..8]) }, true),
        ("CamCal1 zero den", RawTag { ifd: TagIfd::Ifd0, tag: 50723, field_type: 10, count: 9, bytes: srats(&id9.iter().map(|&(n,_)|(n,0)).collect::<Vec<_>>()) }, true),
        ("AnalogBalance 2 entries", RawTag { ifd: TagIfd::Ifd0, tag: 50727, field_type: 5, count: 2, bytes: srats(&[(1,1),(1,1)]) }, true),
        ("AnalogBalance count 0", RawTag { ifd: TagIfd::Ifd0, tag: 50727, field_type: 5, count: 0, bytes: vec![] }, true),
        ("ForwardMatrix1+2, FM2 8 entries", RawTag { ifd: TagIfd::Ifd0, tag: 50965, field_type: 10, count: 8, bytes: srats(&id9[..8]) }, true),
        ("CM1 count 0", RawTag { ifd: TagIfd::Ifd0, tag: 50721, field_type: 10, count: 0, bytes: vec![] }, true),
        ("CM2 count 0", RawTag { ifd: TagIfd::Ifd0, tag: 50722, field_type: 10, count: 0, bytes: vec![] }, true),
        ("CalSig count 0", RawTag { ifd: TagIfd::Ifd0, tag: 50931, field_type: 2, count: 0, bytes: vec![] }, true),
    ];
    for (name, t, cfa) in cases {
        let mut d = desc(cfa);
        if name.starts_with("ForwardMatrix") { d.meta.forward_matrix_1 = Some(id9.clone()); }
        if name.starts_with("CamCal2") || name.starts_with("CM2") { d.meta.color_matrix_2 = d.meta.color_matrix_1.clone(); }
        d.extra_tags.push(t);
        let file = write_dng(&d).unwrap();
        let c = Container::parse(&file).unwrap();
        let s = match c.sensor() { Ok(s) => s, Err(e) => { println!("AUDIT {name}: sensor() Err {e:?}"); continue; } };
        let mut p = vec![0u16; (W * H) as usize];
        unpack_into(&s, c.byte_order(), &file, &mut p).unwrap();
        let r = match output_dimensions(&s) {
            Err(e) => Err(e),
            Ok((w, h)) => if cfa { let mut dst = vec![0u16; (w*h*3) as usize]; develop_color_into(&s, &p, &mut dst) } else { let mut dst = vec![0u16; (w*h) as usize]; develop_into(&s, &p, &mut dst) },
        };
        println!("AUDIT {name}: malformed={:?} result={r:?}", s.malformed_tags);
    }
}
```

## Out of Scope

- A new `Error` variant, any `DEVELOP_VERSION` change, any Tier-B re-pin.
- Changing the public `Container::scalar`, `values` or `array`.
- Refusing a well-formed singular matrix (DEC-078 §4).
- Audit ranks 8 (unknown illuminant code), 9, 11 and 12.
- Commit-message bodies with code tokens: the repo ruleset rejects them. Plain prose only.

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

### Execution notes

- **Branch / PR:** `feat/spec-089-misshapen-tags` → https://github.com/jysf/irradiance/pull/319 (cut from `origin/main` at 48a8310, PR 318's merge; checked with `/usr/bin/git`).
- **Completed at:** 2026-11-23
- **All acceptance criteria met?** yes. AC1 to AC7 by the tests below, AC8 by the corpus run and re-measure, AC9 by the red-proof table.

**Test file.** The build chose a new `tests/misshapen_tags.rs`, reusing `support/dng_writer.rs` and `support/tiff.rs` and the entry / sentinel machinery of `tests/malformed_tags.rs` (copied, not shared: that file keeps SPEC-087's malformed-type rows). The AC1 reader rows run through `Container::parse → sensor → unpack_into` there rather than in `tests/ifd_reader.rs`. AC4 is three unit tests in `src/color.rs`.

**Shown red on main's behaviour first** (tests written before any `src/` change, run at 48a8310 plus the test file): 7 of 9 red.

| Test | On main |
|---|---|
| `ac1_the_reader_records_each_empty_or_misshapen_tag` | red (274 count zero not recorded) |
| `ac1_colour_planes_come_from_the_plane` | red |
| `ac2_each_path_refuses_a_misshapen_tag_it_reads` | red: `274 count zero on DevelopInto: expected MalformedTag { tag: 274 }, got Ok(())` |
| `ac3_a_misshapen_second_colour_matrix_refuses_instead_of_falling_back` | red: `50722 8 entries on DevelopColorInto: … got Ok(())` |
| `ac3_a_misshapen_second_forward_matrix_refuses_instead_of_switching_path` | red: `50965 8 entries on DevelopColorInto: … got Ok(())` |
| `ac5_an_empty_orientation_is_recovered_from_the_other_ifd` | red (count zero in both not recorded) |
| `ac7_the_changed_refusals_are_malformed_tag` | red: `50721 on DevelopColorInto: … got Err(MissingColorCalibration)` |
| `ac1_valid_and_absent_controls_are_not_recorded` | green (control, as expected) |
| `ac6_the_k3iii_exception_and_the_already_refused_tags_hold` | green (exceptions, as expected) |

The AC4 unit tests do not compile on main (`dual_illuminant` returned `Option`), so their red evidence is mutants (d) to (f).

**What changed.**
- `src/ifd.rs`: private `shaped_scalar` (count zero is `Err`; `exactly_one` for `Orientation`, `CFALayout`, `CalibrationIlluminant1/2`) and `ifd0_colour_rationals` (exact count in ColorPlanes, no zero denominator), both next to `cost_the_field`; count-zero `BlackLevel`, `CFAPlaneColor`, both signatures and `BlackLevelDeltaH/V` are `Err`; ColorPlanes is `CFAPlaneColor`'s count on a CFA plane (3 when absent, malformed or the mosaic did not read), `SamplesPerPixel` otherwise (read earlier in `sensor()`, same structural read). `AsShotNeutral` keeps the old read. The public `Container::scalar` is unchanged; its doc comment gains the count-zero sentence. `WhiteLevel` lands once, through `white_level`.
- `src/color.rs`: `dual_illuminant` is `Result<Option<_>, Error>` and propagates a present `ColorMatrix`'s error (absent, unknown code or equal temperatures stay `Ok(None)`); `cam_to_xyz_interpolated` validates each present `ForwardMatrix` and takes the FM path when both are present; `resolve_calibration` validates every present `CameraCalibration`, used or not.
- `CHANGELOG.md` `[Unreleased] → Fixed`: the new refusals and the changed refusals of AC7.

**AC9 red-proof** (scratchpad script applied each mutant to the source, ran the named test, restored the file; `git status` clean after). Green control first: every named test passed unmutated.

| Mutant | Test | Result |
|---|---|---|
| (a) count-zero scalar back to `Ok(None)` | `ac1_the_reader…` (row `274 count zero (cfa false)`), `ac2_each_path…` (`274 count zero on DevelopInto … got Ok(())`) | red, red |
| (b) matrix length check removed | `ac1_the_reader…` (row `50721 count zero`), `ac3_…colour_matrix…` (`50722 8 entries … got Err(MissingColorCalibration)`) | red, red |
| (c) denominator check removed | `ac1_the_reader…` (row `50721 zero denominator in the last entry`), `ac3_…colour_matrix…` (`50722 zero denominator … got Err(MissingColorCalibration)`) | red, red |
| (d) `dual_illuminant` back to `.ok()` | `color::tests::ac4_dual_illuminant_propagates_a_present_matrix_error` | red |
| (e) FM branch back to "both Ok or fall through" | `color::tests::ac4_forward_matrix_error_propagates` | red |
| (f) `resolve_calibration` checks only a used calibration | `color::tests::ac4_unused_calibration_is_validated` | red |
| (g) Orientation recovery dropped (IFD0 error recorded even when the sensor IFD's is valid) | `ac5_an_empty_orientation…` | red |
| (h) length taken as "at least" | `ac1_the_reader…` (row `50721 10 entries`) | red |
| (i) ColorPlanes fixed at 3 | `ac1_colour_planes_come_from_the_plane` | red |
| (j) zero denominator checked on the first entry only | `ac1_the_reader…` (row `50721 zero denominator in the last entry`) | red |

**AC8 corpus re-measure** (`irr ifd`, release build, all 11 held DNGs, before = 48a8310, after = this branch): byte-identical `diff`. Before and after alike: Canon 5D III, Pixel 3a / 4a / 7 Pro, Leica M Monochrom, M Monochrom Typ 246, Q2 Mono ×3 and Nikon D300S `malformed_tags []`; Pentax K-3 III Mono `[50713]`. None of the six mono files carries a colour tag (exiftool), so ColorPlanes = SamplesPerPixel = 1 changes nothing there. `DEVELOP_VERSION` is `"8"`, `tests/error_kind.rs` still counts 76, and no Tier-B pin or fixture changed.

**Gates** (final tree):
- `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images just test`: green, 857 passed, 0 failed across 51 result lines; corpus-status `12/12 present … no tier-B test will skip`.
- `just lint-ci`: green (after a type alias for one test-local type; first run was red on `clippy::type_complexity` in the new test file).
- `just lint-no-allow`, `just fmt-check`, `just doc-check` (2 surfaces, 0 warnings), `just validate`: green (validate prints its standing SPEC-061 advisory).
- **SKIPPED**: `IRRADIANCE_SDK_STAGE2_DIR` and `IRRADIANCE_SDK_COLOUR_DIR` were unset, so the dng_validate stage-2 and SDK colour cross-checks did not run.

### Cost self-report

- **Tokens (total):** 14,259,009 (floor; see `notes`)
- **Estimated USD:** 11.23
- **Duration (minutes):** 18 (transcript span)
- **Source of the number:** this session's transcript, measured per `docs/cost-tracking.md` "Measuring a cycle"

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - Three existing tests encoded the behaviour DEC-078 §7 reverses and were changed, not deleted:
    - `color::tests::degenerate_interpolation_inputs_do_not_panic` case (a): a zero-denominator `ColorMatrix2` with `ForwardMatrix1` used to fall back and develop; it now asserts `MissingColorCalibration`.
    - `tests/color_oracle.rs` `ac3_an_unused_ill_shaped_calibration_develops` (SPEC-085, DEC-076 §3 "only what is used is checked") became `ac3_an_unused_ill_shaped_calibration_refuses`, asserting the refusal on every entry for an unapplied 8-entry `CameraCalibration1` and an unused 8-entry `CameraCalibration2`.
    - `ifd::tests::srational_matrix_reads_signed_and_fractional_verbatim` and `the_five_calibration_tags_read_on_a_writer_fixture` put 9-entry matrices on a 1-sample LinearRaw plane, where §7.1(b) now wants 3; they moved to the existing `cfa_sensor` fixture (3 planes), which keeps what they test.
  - `dual_illuminant` validates both present `ColorMatrix`es before the illuminant check, so a misshapen matrix is an error even with an unknown illuminant code (stricter than the spec's reading for mutant (d); the reader refuses such a file first anyway).
  - A present misshapen `ForwardMatrix1` with `ForwardMatrix2` absent is now an error in the dual path too (it was silently ignored); same §7.2 rule.
- **Follow-up work identified:**
  - DEC-076 §3's text still says "only what is used is checked"; it could get an amendment note pointing at DEC-078 §7.2, as DEC-012 got one for §6.
  - The scan accepts only 1-sample planes, so `SamplesPerPixel` is always 1 on a LinearRaw sensor and ColorPlanes for LinearRaw is always 1; if multi-sample LinearRaw is ever supported, this rule is already in place for it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — That a LinearRaw plane's ColorPlanes is `SamplesPerPixel` = 1 broke two reader unit tests and a SPEC-085 oracle test that the spec did not list; finding them took one full `just test` run each.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-076 §3 (`resolve_calibration` checks only what is used) is directly reversed by §7.2 and has a pinned test in `tests/color_oracle.rs`; naming that test would have saved a gate round.

3. **If you did this task again, what would you do differently?**
   — Grep the existing tests for the old fallbacks (`unused`, `fallback`, matrices on `sensor(&b)`) before the first full run, and run `just lint-ci` on the new test file before the full gate chain.
