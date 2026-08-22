//! `SPEC-005` — shells out to `exiftool` and `dnglab` as ORACLES, run as
//! tools and never linked (`no-copyleft-dependencies`,
//! `provenance-recorded-per-algorithm`). Parses their text output with **no
//! new dependency**: `exiftool -T` needs no parser (split on `\t`, then on
//! `' '`), and `dnglab`'s JSON is read by searching for the handful of keys
//! that are unique in the document, asserting that uniqueness before trusting
//! a match (`docs/oracle-contract.md`, SPEC-005 Implementation Context).
//!
//! This is test-support code, `#[path]`-included by `tests/metadata_oracle.rs`
//! only. It is not part of the library and is never compiled into it.
//!
//! ⚠ `dnglab`'s exit code carries real signal (it exits 2 on a truncated
//! file); `exiftool`'s does not (it exits 0 on a truncated file and on an
//! absent tag — only the values do). The two tool wrappers below treat exit
//! status accordingly: `dnglab_meta` checks it, `exiftool`/`exiftool_reading`
//! never do.

#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Output};

use irradiance::ifd::{ActiveArea, DefaultCropOrigin, DefaultCropSize, Sensor};

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

/// Why a tool reading could not be produced. AC6: a missing tool is this
/// variant, named — never a bare process-spawn error a caller has to decode.
#[derive(Debug)]
pub enum ToolError {
    /// The tool did not resolve on PATH (`std::io::ErrorKind::NotFound` from
    /// the spawn attempt itself — the OS's own signal, not a heuristic probe).
    NotOnPath(&'static str),
    /// The tool ran but its output could not be used (non-zero exit where
    /// that carries signal, or a spawn failure that was not `NotFound`).
    Exec { tool: &'static str, message: String },
    /// The tool's stdout did not have the shape this reader expects.
    Parse { tool: &'static str, message: String },
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::NotOnPath(tool) => write!(f, "{tool} is not on PATH"),
            ToolError::Exec { tool, message } => write!(f, "{tool}: {message}"),
            ToolError::Parse { tool, message } => {
                write!(f, "{tool}: could not parse its output — {message}")
            }
        }
    }
}

/// Spawn `bin` with `args` then `path` as the final argument, classifying a
/// spawn failure precisely: `NotFound` (ENOENT — the binary is not on PATH)
/// becomes [`ToolError::NotOnPath`], anything else [`ToolError::Exec`]. Both
/// `exiftool()` and `dnglab_analyze_meta()` go through this one place, so
/// AC6 ("missing tool skips loudly, naming the tool") is one guard, not two.
pub fn run_tool(bin: &'static str, args: &[String], path: &Path) -> Result<Output, ToolError> {
    Command::new(bin)
        .args(args)
        .arg(path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ToolError::NotOnPath(bin)
            } else {
                ToolError::Exec {
                    tool: bin,
                    message: e.to_string(),
                }
            }
        })
}

/// Whether `exiftool` resolves on PATH — for a caller that wants to skip
/// loudly *before* attempting real work, rather than via the `Err` path.
pub fn exiftool_available() -> bool {
    Command::new("exiftool").arg("-ver").output().is_ok()
}

/// Whether `dnglab` resolves on PATH.
pub fn dnglab_available() -> bool {
    Command::new("dnglab").arg("--version").output().is_ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// exiftool
// ─────────────────────────────────────────────────────────────────────────────

/// One `-T -n -s3` field, alongside the fully-qualified tag that produced it
/// (e.g. `"SubIFD:ImageWidth"`), so a mismatch can name which field
/// disagreed. `None` is exiftool's `-` — a tag reported absent (AC2).
#[derive(Debug, Clone)]
pub struct Field {
    pub tag: String,
    pub values: Option<Vec<u32>>,
}

/// Run `exiftool -T -n -s3 -{group}:{tag}...` and return one [`Field`] per
/// requested tag, in the order requested.
///
/// `-T` tab-separated, `-n` numeric (without it `Orientation` prints
/// `"Horizontal (normal)"`), `-s3` bare values with `-` for absent — SPEC-005
/// Implementation Context. exiftool's exit code carries **no signal** (it
/// exits 0 on a truncated file and on an absent tag); only stdout is trusted.
pub fn exiftool(path: &Path, group: &str, tags: &[&str]) -> Result<Vec<Field>, ToolError> {
    let qualified: Vec<String> = tags.iter().map(|t| format!("{group}:{t}")).collect();
    let output = run_exiftool(path, &qualified)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_fields(&qualified, &stdout)
}

fn run_exiftool(path: &Path, qualified_tags: &[String]) -> Result<Output, ToolError> {
    let mut args: Vec<String> = vec!["-T".into(), "-n".into(), "-s3".into()];
    args.extend(qualified_tags.iter().map(|t| format!("-{t}")));
    run_tool("exiftool", &args, path)
}

/// Parse one `-T -n -s3` output line into [`Field`]s, in `qualified_tags`'
/// order. Pure — shared by [`exiftool`]/[`exiftool_reading`] (the real tool)
/// and `tests/oracle-fixtures/` (AC5 tier A: the SAME parsing code, replayed
/// against committed text, with no tool installed).
pub fn parse_fields(qualified_tags: &[String], line: &str) -> Result<Vec<Field>, ToolError> {
    let line = line.trim_end_matches('\n');
    let raw: Vec<&str> = line.split('\t').collect();
    if raw.len() != qualified_tags.len() {
        return Err(ToolError::Parse {
            tool: "exiftool",
            message: format!(
                "expected {} tab-separated field(s) for {qualified_tags:?}, got {}: {line:?}",
                qualified_tags.len(),
                raw.len()
            ),
        });
    }
    Ok(qualified_tags
        .iter()
        .zip(raw)
        .map(|(tag, r)| Field {
            tag: tag.clone(),
            values: parse_field_values(r),
        })
        .collect())
}

fn parse_field_values(raw: &str) -> Option<Vec<u32>> {
    if raw == "-" {
        None
    } else {
        Some(
            raw.split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().expect("exiftool -n emits plain integers"))
                .collect(),
        )
    }
}

/// Look up a field by its bare tag name (the part after the group prefix).
pub fn values_for<'a>(fields: &'a [Field], bare_tag: &str) -> Option<&'a Vec<u32>> {
    fields
        .iter()
        .find(|f| f.tag.rsplit(':').next() == Some(bare_tag))
        .and_then(|f| f.values.as_ref())
}

/// The eleven sensor-IFD tags [`exiftool_reading`] requests, in a fixed order
/// shared with `tests/oracle-fixtures/`.
const SENSOR_TAGS: &[&str] = &[
    "ImageWidth",
    "ImageHeight",
    "BitsPerSample",
    "Compression",
    "PhotometricInterpretation",
    "BlackLevel",
    "WhiteLevel",
    "BlackLevelRepeatDim",
    "ActiveArea",
    "DefaultCropOrigin",
    "DefaultCropSize",
];

/// Every tag [`exiftool_reading`] requests: `SENSOR_TAGS` qualified by
/// `group`, plus `IFD0:Orientation` — Orientation lives in `IFD0` on all
/// seven corpus files, never the sensor IFD (SPEC-005 Implementation
/// Context), and the group naming a file's sensor IFD is measured per-file,
/// never derived (see `tests/metadata_oracle.rs`).
pub fn sensor_reading_tags(group: &str) -> Vec<String> {
    let mut tags: Vec<String> = SENSOR_TAGS.iter().map(|t| format!("{group}:{t}")).collect();
    tags.push("IFD0:Orientation".to_string());
    tags
}

/// Everything `exiftool` says about one sensor IFD, typed to compare
/// directly against `irradiance::ifd::Sensor` ([`diff`]).
#[derive(Debug, Clone)]
pub struct ToolReading {
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: u32,
    pub compression: u32,
    pub photometric: u32,
    pub black_level: Option<u32>,
    pub white_level: Option<u32>,
    pub black_level_repeat_dim: Option<[u32; 2]>,
    pub active_area: Option<ActiveArea>,
    pub default_crop_origin: Option<DefaultCropOrigin>,
    pub default_crop_size: Option<DefaultCropSize>,
    pub orientation: Option<u32>,
}

/// Run `exiftool` over the sensor IFD (`group`, e.g. `"SubIFD"` or `"IFD0"`)
/// plus `IFD0:Orientation`, and return a typed [`ToolReading`].
pub fn exiftool_reading(path: &Path, group: &str) -> Result<ToolReading, ToolError> {
    let qualified = sensor_reading_tags(group);
    let output = run_exiftool(path, &qualified)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    reading_from_fields(&parse_fields(&qualified, &stdout)?)
}

/// Build a [`ToolReading`] from parsed fields — shared by [`exiftool_reading`]
/// (the real tool) and the tier-A fixture test (AC5), so the red-proof
/// exercises the exact parsing code a real run would use.
pub fn reading_from_fields(fields: &[Field]) -> Result<ToolReading, ToolError> {
    let req = |tag: &str| -> Result<u32, ToolError> {
        values_for(fields, tag)
            .and_then(|v| v.first().copied())
            .ok_or_else(|| ToolError::Parse {
                tool: "exiftool",
                message: format!("{tag} is required but exiftool reported it absent"),
            })
    };
    let opt = |tag: &str| values_for(fields, tag).and_then(|v| v.first().copied());

    Ok(ToolReading {
        width: req("ImageWidth")?,
        height: req("ImageHeight")?,
        bits_per_sample: req("BitsPerSample")?,
        compression: req("Compression")?,
        photometric: req("PhotometricInterpretation")?,
        black_level: opt("BlackLevel"),
        white_level: opt("WhiteLevel"),
        black_level_repeat_dim: values_for(fields, "BlackLevelRepeatDim")
            .and_then(|v| <[u32; 2]>::try_from(v.as_slice()).ok()),
        active_area: values_for(fields, "ActiveArea").and_then(|v| match v.as_slice() {
            [top, left, bottom, right] => Some(ActiveArea {
                top: *top,
                left: *left,
                bottom: *bottom,
                right: *right,
            }),
            _ => None,
        }),
        default_crop_origin: values_for(fields, "DefaultCropOrigin").and_then(|v| {
            match v.as_slice() {
                [x, y] => Some(DefaultCropOrigin { x: *x, y: *y }),
                _ => None,
            }
        }),
        default_crop_size: values_for(fields, "DefaultCropSize").and_then(|v| match v.as_slice() {
            [width, height] => Some(DefaultCropSize {
                width: *width,
                height: *height,
            }),
            _ => None,
        }),
        orientation: opt("Orientation"),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// The comparator — AC1/AC2/AC5
// ─────────────────────────────────────────────────────────────────────────────

/// One field where our reading and the tool's disagreed. Named, not
/// "mismatch" (AC1: "naming the file, the field, ours and theirs").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    pub field: &'static str,
    pub ours: String,
    pub theirs: String,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: ours={}, theirs={}",
            self.field, self.ours, self.theirs
        )
    }
}

/// Compare a container's [`Sensor`] against exiftool's reading of the same
/// IFD, field by field (AC1). **Every one of the eleven fields is compared
/// unconditionally** — there is no exemption for a tag recorded in
/// `Sensor::malformed_tags`.
///
/// `SPEC-005` shipped with one (`DEC-013`, now `rejected`) and it was dead
/// code: removing it left all 21 oracle tests green, because the case it
/// suppressed cannot currently arise. `exiftool` reports `K3III.DNG`'s
/// malformed `BlackLevelRepeatDim` as a bare `1`,
/// [`reading_from_fields`]'s `<[u32; 2]>::try_from(..).ok()` degrades that
/// to `None`, and `DEC-012` independently gives us `None` — so the two
/// already agree and there is nothing to exempt.
///
/// ⚠ **That agreement is an accident of `SPEC-005/FU-1`**, the defect where
/// a shape-odd tool value is reclassified as absence. The day `FU-1` is
/// fixed, `K3III.DNG` goes red here — and that is deliberate. Whoever fixes
/// `FU-1` must then decide, with a test, whether a `DEC-012`-tolerated tag
/// is exempt from this diff. Leaving the guard in place would have let that
/// decision happen silently, by absorption, which is the one outcome
/// `DEC-013` was trying to avoid and the one it would have caused.
///
/// That alarm is **measured, not reasoned** (2026-08-22): with the `FU-1`
/// fix simulated here — a one-element reading mapped to `Some([a, a])`
/// instead of `None` — `metadata_matches_exiftool_on_every_corpus_file`
/// fails immediately with
/// `PENTAX-K3III-MONO/K3III.DNG: BlackLevelRepeatDim: ours=None,
/// theirs=Some([1, 1])`. Mutation asserted applied and compiled; tree
/// restored byte-identical.
pub fn diff(sensor: &Sensor, reading: &ToolReading) -> Vec<Mismatch> {
    let mut out = Vec::new();

    if (sensor.width, sensor.height) != (reading.width, reading.height) {
        out.push(Mismatch {
            field: "dimensions",
            ours: format!("{}x{}", sensor.width, sensor.height),
            theirs: format!("{}x{}", reading.width, reading.height),
        });
    }
    if sensor.bits_per_sample != reading.bits_per_sample {
        out.push(Mismatch {
            field: "BitsPerSample",
            ours: sensor.bits_per_sample.to_string(),
            theirs: reading.bits_per_sample.to_string(),
        });
    }
    if sensor.compression.code() != reading.compression {
        out.push(Mismatch {
            field: "Compression",
            ours: sensor.compression.code().to_string(),
            theirs: reading.compression.to_string(),
        });
    }
    if sensor.photometric != reading.photometric {
        out.push(Mismatch {
            field: "PhotometricInterpretation",
            ours: sensor.photometric.to_string(),
            theirs: reading.photometric.to_string(),
        });
    }
    if sensor.black_level != reading.black_level {
        out.push(Mismatch {
            field: "BlackLevel",
            ours: format!("{:?}", sensor.black_level),
            theirs: format!("{:?}", reading.black_level),
        });
    }
    if sensor.white_level != reading.white_level {
        out.push(Mismatch {
            field: "WhiteLevel",
            ours: format!("{:?}", sensor.white_level),
            theirs: format!("{:?}", reading.white_level),
        });
    }
    if sensor.black_level_repeat_dim != reading.black_level_repeat_dim {
        out.push(Mismatch {
            field: "BlackLevelRepeatDim",
            ours: format!("{:?}", sensor.black_level_repeat_dim),
            theirs: format!("{:?}", reading.black_level_repeat_dim),
        });
    }
    if sensor.active_area != reading.active_area {
        out.push(Mismatch {
            field: "ActiveArea",
            ours: format!("{:?}", sensor.active_area),
            theirs: format!("{:?}", reading.active_area),
        });
    }
    if sensor.default_crop_origin != reading.default_crop_origin {
        out.push(Mismatch {
            field: "DefaultCropOrigin",
            ours: format!("{:?}", sensor.default_crop_origin),
            theirs: format!("{:?}", reading.default_crop_origin),
        });
    }
    if sensor.default_crop_size != reading.default_crop_size {
        out.push(Mismatch {
            field: "DefaultCropSize",
            ours: format!("{:?}", sensor.default_crop_size),
            theirs: format!("{:?}", reading.default_crop_size),
        });
    }
    if sensor.orientation != reading.orientation {
        out.push(Mismatch {
            field: "Orientation",
            ours: format!("{:?}", sensor.orientation),
            theirs: format!("{:?}", reading.orientation),
        });
    }

    out
}

// ─────────────────────────────────────────────────────────────────────────────
// dnglab
// ─────────────────────────────────────────────────────────────────────────────

/// The six scalars AC3 cross-checks, plus `cropArea.p` (kept separately —
/// AC4.1 compares it arithmetically, not directly, because it is
/// sensor-absolute where ours and exiftool's are DNG-relative).
#[derive(Debug, Clone)]
pub struct DnglabMeta {
    pub raw_width: u32,
    pub raw_height: u32,
    pub bit_depth: u32,
    pub white_level: u32,
    pub orientation: u32,
    pub black_level: u32,
    /// `cropArea.p`: sensor-absolute `(x, y)` — SPEC-005 AC4.1.
    pub crop_area_p: (u32, u32),
}

/// The raw stdout/stderr/exit status of one `dnglab analyze --meta --json`
/// run — exposed separately from [`dnglab_meta`] so AC4.3 can assert on the
/// ANSI warning dnglab writes to **stderr** for `K3III.DNG`, which is never
/// merged into stdout here (`Command::output` keeps them separate streams by
/// construction — the fix for the `2>&1` trap SPEC-005 measured).
pub struct DnglabRun {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub fn dnglab_analyze_meta(path: &Path) -> Result<DnglabRun, ToolError> {
    let args: Vec<String> = ["analyze", "--meta", "--json"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let output = run_tool("dnglab", &args, path)?;
    Ok(DnglabRun {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Run `dnglab analyze --meta --json` and parse the six scalars plus
/// `cropArea.p`. Unlike `exiftool`, dnglab's exit code carries real signal
/// (it exits 2 on a truncated file), so a non-zero exit is a loud
/// [`ToolError::Exec`], not silently parsed anyway.
pub fn dnglab_meta(path: &Path) -> Result<DnglabMeta, ToolError> {
    let run = dnglab_analyze_meta(path)?;
    if !run.success {
        return Err(ToolError::Exec {
            tool: "dnglab",
            message: format!(
                "analyze --meta --json exited unsuccessfully: {}",
                run.stderr
            ),
        });
    }
    parse_dnglab_meta(&run.stdout)
}

/// Parse the subset of `dnglab analyze --meta --json`'s output this oracle
/// reads. Pure — shared by [`dnglab_meta`] and (potentially) a fixture test,
/// same reasoning as [`parse_fields`] above.
pub fn parse_dnglab_meta(json: &str) -> Result<DnglabMeta, ToolError> {
    Ok(DnglabMeta {
        raw_width: unique_scalar_u32(json, "rawWidth")?,
        raw_height: unique_scalar_u32(json, "rawHeight")?,
        bit_depth: unique_scalar_u32(json, "bitDepth")?,
        white_level: unique_array_first_u32(json, "whitelevels")?,
        orientation: unique_scalar_u32(json, "orientation")?,
        black_level: black_level_scalar(json)?,
        crop_area_p: extract_crop_area_p(json)?,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Ad hoc JSON scalar extraction — no dependency (SPEC-005 AC7)
//
// `rawWidth`, `rawHeight`, `bitDepth`, `whitelevels`, `orientation` and
// `levels` (under `blacklevels`) are each unique KEYS in the whole document
// (measured 2026-08-21) — every extraction below asserts that count is
// exactly 1 before trusting a match, the same discipline
// `attribute-text-inside-doc-comments` states for source text: a bare
// substring search finds a value as readily as prose about it.
//
// `x`/`y`/`w`/`h` are NOT unique (they appear under both `cropArea` and
// `activeArea`) — `extract_crop_area_p` scopes the search to the brace-matched
// `cropArea` object first, where they ARE unique.
// ─────────────────────────────────────────────────────────────────────────────

fn unique_value_start(json: &str, key: &str) -> Result<usize, ToolError> {
    let needle = format!("\"{key}\":");
    let count = json.matches(needle.as_str()).count();
    if count != 1 {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!(
                "{key:?} occurs {count} time(s) in the JSON document (expected exactly 1) — \
                 the uniqueness this extraction depends on no longer holds"
            ),
        });
    }
    let at = json
        .find(needle.as_str())
        .expect("just counted exactly one occurrence");
    Ok(at + needle.len())
}

fn scalar_str(json: &str, start: usize) -> &str {
    let rest = json[start..].trim_start();
    let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
    rest[..end].trim()
}

fn unique_scalar_u32(json: &str, key: &str) -> Result<u32, ToolError> {
    let start = unique_value_start(json, key)?;
    let raw = scalar_str(json, start);
    raw.parse().map_err(|_| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key:?} = {raw:?} is not an integer"),
    })
}

fn unique_array_str<'a>(json: &'a str, key: &str) -> Result<&'a str, ToolError> {
    let start = unique_value_start(json, key)?;
    let rest = json[start..].trim_start();
    if !rest.starts_with('[') {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!("{key:?} is not a JSON array"),
        });
    }
    let close = rest.find(']').ok_or_else(|| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key:?}: unterminated array"),
    })?;
    Ok(&rest[1..close])
}

fn unique_array_first_u32(json: &str, key: &str) -> Result<u32, ToolError> {
    let inner = unique_array_str(json, key)?;
    let items: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if items.len() != 1 {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!("{key:?} has {} element(s), expected exactly 1", items.len()),
        });
    }
    items[0].parse().map_err(|_| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key:?}[0] = {:?} is not an integer", items[0]),
    })
}

/// `blacklevels.levels` is an array of rational STRINGS (`"512/1"`, not
/// `512`). Parse `N/D`; do NOT assume `D == 1` (SPEC-005 Implementation
/// Context) — a non-integer ratio is a genuine fourth divergence to report,
/// not something to round away.
fn black_level_scalar(json: &str) -> Result<u32, ToolError> {
    let inner = unique_array_str(json, "levels")?;
    let items: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if items.len() != 1 {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!(
                "blacklevels.levels has {} element(s), expected exactly 1 (SamplesPerPixel 1)",
                items.len()
            ),
        });
    }
    parse_rational_u32("blacklevels.levels[0]", items[0])
}

fn parse_rational_u32(key: &str, raw: &str) -> Result<u32, ToolError> {
    let raw = raw.trim_matches('"');
    let (n, d) = raw.split_once('/').ok_or_else(|| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key}: {raw:?} is not an N/D rational string"),
    })?;
    let n: u64 = n.parse().map_err(|_| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key}: {raw:?} — numerator is not an integer"),
    })?;
    let d: u64 = d.parse().map_err(|_| ToolError::Parse {
        tool: "dnglab",
        message: format!("{key}: {raw:?} — denominator is not an integer"),
    })?;
    if d == 0 || !n.is_multiple_of(d) {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!(
                "{key}: {raw:?} is not an even ratio — SPEC-005 assumed D==1 or N%D==0; this \
                 is a fourth divergence to report, not a parse bug to paper over"
            ),
        });
    }
    Ok((n / d) as u32)
}

/// Byte range of `"key":{...}` — brace-depth aware, so it does not stop at
/// the first nested `}` (`cropArea` contains both `p` and `d` objects).
/// `Ok(None)` when the value is JSON `null` (an absent `activeArea`, e.g. on
/// `M2462362.DNG` — not used by this oracle today, but `cropArea` itself is
/// never null on a DNG, so this only matters if that changes).
fn unique_object_str<'a>(json: &'a str, key: &str) -> Result<Option<&'a str>, ToolError> {
    let start = unique_value_start(json, key)?;
    let rest = json[start..].trim_start();
    if rest.starts_with("null") {
        return Ok(None);
    }
    if !rest.starts_with('{') {
        return Err(ToolError::Parse {
            tool: "dnglab",
            message: format!("{key:?} is neither an object nor null"),
        });
    }
    let mut depth = 0i32;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(Some(&rest[..=i]));
                }
            }
            _ => {}
        }
    }
    Err(ToolError::Parse {
        tool: "dnglab",
        message: format!("{key:?}: unterminated object"),
    })
}

fn extract_crop_area_p(json: &str) -> Result<(u32, u32), ToolError> {
    let crop = unique_object_str(json, "cropArea")?.ok_or_else(|| ToolError::Parse {
        tool: "dnglab",
        message: "cropArea was null".to_string(),
    })?;
    let x = unique_scalar_u32(crop, "x")?;
    let y = unique_scalar_u32(crop, "y")?;
    Ok((x, y))
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — the ad hoc JSON parser's error paths. The real corpus only
// ever exercises the happy path; these are the malformed/edge shapes a line
// counter cannot see, and every new function here gets at least one
// (AGENTS.md §12).
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_unique_key_is_rejected_not_taken_first() {
        let json = r#"{"x":1,"nested":{"x":2}}"#;
        let err =
            unique_scalar_u32(json, "x").expect_err("x occurs twice — must not silently pick one");
        assert!(matches!(err, ToolError::Parse { tool: "dnglab", .. }));
        assert!(err.to_string().contains("2 time"));
    }

    #[test]
    fn a_missing_key_is_a_named_parse_error() {
        let err = unique_scalar_u32("{}", "rawWidth").expect_err("key is absent");
        assert!(err.to_string().contains("rawWidth"));
    }

    #[test]
    fn a_non_integer_scalar_is_rejected() {
        let err = unique_scalar_u32(r#"{"bitDepth":"fourteen"}"#, "bitDepth")
            .expect_err("a quoted string is not an integer");
        assert!(err.to_string().contains("bitDepth"));
    }

    #[test]
    fn whitelevels_with_more_than_one_element_is_rejected() {
        let err = unique_array_first_u32(r#"{"whitelevels":[1,2]}"#, "whitelevels")
            .expect_err("two elements — this oracle only trusts a single-element array");
        assert!(err.to_string().contains('2'));
    }

    #[test]
    fn a_rational_with_zero_denominator_is_rejected() {
        let err = parse_rational_u32("levels[0]", "\"64/0\"").expect_err("D == 0");
        assert!(matches!(err, ToolError::Parse { .. }));
    }

    #[test]
    fn a_rational_that_does_not_divide_evenly_is_a_fourth_divergence_not_rounded() {
        // SPEC-005 Implementation Context: "do not assume D == 1 without
        // checking" — a genuine non-integer ratio must be a loud Err, never
        // silently truncated to an integer.
        let err = parse_rational_u32("levels[0]", "\"5/2\"").expect_err("5/2 is not an integer");
        assert!(err.to_string().contains("even ratio"));
    }

    #[test]
    fn a_rational_that_divides_evenly_with_a_non_one_denominator_is_accepted() {
        // D == 1 is the only case measured on the real corpus, but the
        // parser's contract is N % D == 0, not D == 1 specifically.
        let value = parse_rational_u32("levels[0]", "\"1024/2\"").expect("1024/2 == 512");
        assert_eq!(value, 512);
    }

    #[test]
    fn crop_area_p_is_scoped_to_the_croparea_object_not_the_whole_document() {
        // activeArea.p ALSO has an "x"/"y" — if extraction were not
        // brace-scoped to cropArea specifically, this would either pick the
        // wrong one or fail the whole-document uniqueness check.
        let json = r#"{
            "cropArea": {"p": {"x": 12, "y": 24}, "d": {"w": 8368, "h": 5584}},
            "activeArea": {"p": {"x": 0, "y": 0}, "d": {"w": 8392, "h": 5632}}
        }"#;
        assert_eq!(extract_crop_area_p(json).expect("parses"), (12, 24));
    }

    #[test]
    fn a_null_crop_area_is_a_named_error_not_a_panic() {
        let err = extract_crop_area_p(r#"{"cropArea": null}"#).expect_err("cropArea is null");
        assert!(err.to_string().contains("cropArea"));
    }

    #[test]
    fn parse_fields_rejects_a_field_count_mismatch() {
        let tags = vec![
            "SubIFD:ImageWidth".to_string(),
            "SubIFD:ImageHeight".to_string(),
        ];
        let err = parse_fields(&tags, "8424").expect_err("one tab field for two requested tags");
        assert!(err.to_string().contains("expected 2"));
    }

    #[test]
    fn parse_fields_reads_a_dash_as_absent() {
        let tags = vec!["SubIFD:BlackLevel".to_string()];
        let fields = parse_fields(&tags, "-").expect("parses");
        assert_eq!(fields[0].values, None);
    }

    #[test]
    fn values_for_matches_the_bare_tag_not_a_suffix_collision() {
        let fields = vec![
            Field {
                tag: "SubIFD:BlackLevel".to_string(),
                values: Some(vec![512]),
            },
            Field {
                tag: "SubIFD:BlackLevelRepeatDim".to_string(),
                values: Some(vec![1, 1]),
            },
        ];
        assert_eq!(values_for(&fields, "BlackLevel"), Some(&vec![512]));
        assert_eq!(
            values_for(&fields, "BlackLevelRepeatDim"),
            Some(&vec![1, 1])
        );
    }
}
