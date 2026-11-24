//! Oracle helpers shared by `tests/dng_write.rs` (tier A) and
//! `tests/dng_write_corpus.rs` (tier B) — `SPEC-061` §7.
//!
//! Oracle 1 (the round trip) is [`check_round_trip`]: parse the written file
//! with irradiance's own reader, unpack its plane, and compare samples and
//! every `Sensor` field `SPEC-061` §3/§4 fixes against [`Expected`], which is
//! derived from the inputs by the framing table, independently of the writer's
//! code. It returns the mismatches rather than asserting, so the red-proof
//! (oracle 3) can show it turning red.
#![allow(dead_code)]

use irradiance::dng::write::{Framing, WriteOptions};
use irradiance::ifd::{ActiveArea, Container, DefaultCropOrigin, DefaultCropSize, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;

/// Parse the file, pick its sensor, unpack it.
pub fn read_back(file: &[u8]) -> Result<(Sensor, Vec<u16>), Error> {
    let container = Container::parse(file)?;
    let sensor = container.sensor()?;
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), file, &mut plane)?;
    Ok((sensor, plane))
}

/// The samples inside `area` of a row-major `width`-wide plane.
pub fn cut(samples: &[u16], width: u32, area: ActiveArea) -> Vec<u16> {
    let w = width as usize;
    let mut out = Vec::new();
    for row in area.top as usize..area.bottom as usize {
        let start = row * w;
        out.extend_from_slice(&samples[start + area.left as usize..start + area.right as usize]);
    }
    out
}

/// What the written file must read back as.
#[derive(Debug, Clone, PartialEq)]
pub struct Expected {
    pub width: u32,
    pub height: u32,
    pub samples: Vec<u16>,
    pub black_levels: Vec<u32>,
    pub black_level_repeat_dim: Option<[u32; 2]>,
    pub white_levels: Vec<u32>,
    pub active_area: Option<ActiveArea>,
    pub masked_areas: Vec<ActiveArea>,
    pub crop_origin: DefaultCropOrigin,
    pub crop_size: DefaultCropSize,
    pub orientation: u32,
    pub opcode_lists: [Option<Vec<u8>>; 3],
}

/// [`Expected`] from the writer's inputs, by `SPEC-061` §3's table.
pub fn expected(samples: &[u16], width: u32, height: u32, o: &WriteOptions<'_>) -> Expected {
    let whole = ActiveArea {
        top: 0,
        left: 0,
        bottom: height,
        right: width,
    };
    let aa = o.active_area.unwrap_or(whole);
    let (aw, ah) = (aa.right - aa.left, aa.bottom - aa.top);
    let full_crop = (
        DefaultCropOrigin { x: 0, y: 0 },
        DefaultCropSize {
            width: aw,
            height: ah,
        },
    );
    let (plane, w, h, active, masked) = match o.framing {
        Framing::FullSensor => (
            samples.to_vec(),
            width,
            height,
            o.active_area,
            o.masked_areas.to_vec(),
        ),
        _ => (cut(samples, width, aa), aw, ah, None, Vec::new()),
    };
    let (crop_origin, crop_size) = match o.framing {
        Framing::StandardCrop => o.default_crop.unwrap_or(full_crop),
        _ => full_crop,
    };
    Expected {
        width: w,
        height: h,
        samples: plane,
        black_levels: o.black_level.to_vec(),
        black_level_repeat_dim: o
            .black_level_repeat_dim
            .map(|[r, c]| [u32::from(r), u32::from(c)]),
        white_levels: vec![o.white_level],
        active_area: active,
        masked_areas: masked,
        crop_origin,
        crop_size,
        orientation: u32::from(o.orientation),
        opcode_lists: [
            o.opcode_list_1.map(<[u8]>::to_vec),
            o.opcode_list_2.map(<[u8]>::to_vec),
            o.opcode_list_3.map(<[u8]>::to_vec),
        ],
    }
}

/// Oracle 1. `Ok(())` when the file reads back as `want`; otherwise every
/// mismatch, named.
pub fn check_round_trip(file: &[u8], want: &Expected) -> Result<(), Vec<String>> {
    let (sensor, plane) = read_back(file).map_err(|e| vec![format!("read back failed: {e}")])?;
    let mut bad = Vec::new();
    let mut check = |name: &str, ok: bool, detail: String| {
        if !ok {
            bad.push(format!("{name}: {detail}"));
        }
    };
    check(
        "dims",
        (sensor.width, sensor.height) == (want.width, want.height),
        format!(
            "got {}x{}, want {}x{}",
            sensor.width, sensor.height, want.width, want.height
        ),
    );
    check(
        "bits",
        sensor.bits_per_sample == 16,
        format!("got {}", sensor.bits_per_sample),
    );
    if plane != want.samples {
        let first = plane
            .iter()
            .zip(&want.samples)
            .position(|(a, b)| a != b)
            .unwrap_or(plane.len().min(want.samples.len()));
        check(
            "samples",
            false,
            format!(
                "differ from index {first} ({} vs {} samples)",
                plane.len(),
                want.samples.len()
            ),
        );
    }
    check(
        "black_levels",
        sensor.black_levels == want.black_levels,
        format!(
            "got {:?}, want {:?}",
            sensor.black_levels, want.black_levels
        ),
    );
    check(
        "black_level_repeat_dim",
        sensor.black_level_repeat_dim == want.black_level_repeat_dim,
        format!(
            "got {:?}, want {:?}",
            sensor.black_level_repeat_dim, want.black_level_repeat_dim
        ),
    );
    check(
        "white_levels",
        sensor.white_levels == want.white_levels,
        format!(
            "got {:?}, want {:?}",
            sensor.white_levels, want.white_levels
        ),
    );
    check(
        "active_area",
        sensor.active_area == want.active_area,
        format!("got {:?}, want {:?}", sensor.active_area, want.active_area),
    );
    check(
        "masked_areas",
        sensor.masked_areas == want.masked_areas,
        format!(
            "got {:?}, want {:?}",
            sensor.masked_areas, want.masked_areas
        ),
    );
    check(
        "default_crop_origin",
        sensor.default_crop_origin == Some(want.crop_origin),
        format!(
            "got {:?}, want {:?}",
            sensor.default_crop_origin, want.crop_origin
        ),
    );
    check(
        "default_crop_size",
        sensor.default_crop_size == Some(want.crop_size),
        format!(
            "got {:?}, want {:?}",
            sensor.default_crop_size, want.crop_size
        ),
    );
    check(
        "orientation",
        sensor.orientation == Some(want.orientation),
        format!("got {:?}, want {}", sensor.orientation, want.orientation),
    );
    let lists = [
        &sensor.opcode_list_1,
        &sensor.opcode_list_2,
        &sensor.opcode_list_3,
    ];
    for (i, (got, want)) in lists.iter().zip(&want.opcode_lists).enumerate() {
        check(
            &format!("opcode_list_{}", i + 1),
            *got == want,
            format!(
                "got {:?} bytes, want {:?} bytes",
                got.as_ref().map(Vec::len),
                want.as_ref().map(Vec::len)
            ),
        );
    }
    check(
        "malformed_tags",
        sensor.malformed_tags.is_empty(),
        format!("got {:?}", sensor.malformed_tags),
    );
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad)
    }
}

/// The writer's options for re-writing a parsed source at `framing`: its own
/// levels, geometry, orientation and opcode lists, and a model name.
pub fn options_from_sensor<'a>(
    sensor: &'a Sensor,
    framing: Framing,
    model: &'a str,
) -> WriteOptions<'a> {
    let mut o = WriteOptions::default();
    o.framing = framing;
    o.active_area = sensor.active_area;
    o.default_crop = sensor.default_crop_origin.zip(sensor.default_crop_size);
    o.orientation = sensor.orientation.map_or(1, |v| v as u16);
    o.black_level = &sensor.black_levels;
    o.black_level_repeat_dim = sensor
        .black_level_repeat_dim
        .map(|[r, c]| [r as u16, c as u16]);
    // A source without WhiteLevel means 2^bits - 1 (DNG); the written file is
    // 16-bit, so that default must become an explicit tag (SPEC-061 §4).
    o.white_level = sensor
        .white_level
        .unwrap_or((1u32 << sensor.bits_per_sample) - 1);
    if matches!(framing, Framing::FullSensor) {
        o.masked_areas = &sensor.masked_areas;
    }
    o.opcode_list_1 = sensor.opcode_list_1.as_deref();
    o.opcode_list_2 = sensor.opcode_list_2.as_deref();
    o.opcode_list_3 = sensor.opcode_list_3.as_deref();
    o.unique_camera_model = Some(model);
    o
}
