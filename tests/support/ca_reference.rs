//! The test-side reference for `SPEC-099`'s CA step (`DEC-087` item 3),
//! written from the decision's text and calling no crate code: the
//! 4096-interval table of `f_c/f_g` over `σ = ρ²` with its nodes solved by
//! bisection, the ratio held at the rim beyond green's image, the source
//! `c + k·(x − c)`, and `DEC-024`'s bilinear kernel clamped to the
//! `ActiveArea` and rounded half up.
//!
//! Shared by `tests/plane_warp.rs` (tier A), `tests/leica_colour.rs` (AC7) and
//! `tests/colour_sdk_stage3.rs` (AC8), so the three oracles check one rule.
#![allow(dead_code)]

/// `T`, the table's interval count.
pub const T: usize = 4096;

/// `f(k, s) = k0 + k1·s + k2·s·s + k3·s·s·s`, in this operation order.
pub fn f(k: [f64; 4], s: f64) -> f64 {
    k[0] + k[1] * s + k[2] * s * s + k[3] * s * s * s
}

/// The node `s_j` for `σ_j`: 1 at and beyond green's rim, else bisection of
/// `h(s) = s·f_g(s)·f_g(s)` against `σ`, stopping when the midpoint is an end.
pub fn node(g: [f64; 4], sigma: f64) -> f64 {
    let rim = f(g, 1.0);
    if sigma >= rim * rim {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    loop {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi {
            break;
        }
        let fm = f(g, mid);
        if mid * fm * fm < sigma {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// `t_c[j] = f_c(s_j)/f_g(s_j)`, `j = 0..=T`.
pub fn table(c: [f64; 4], g: [f64; 4]) -> Vec<f64> {
    (0..=T)
        .map(|j| {
            let s = node(g, j as f64 / T as f64);
            f(c, s) / f(g, s)
        })
        .collect()
}

/// `DEC-024`'s bilinear kernel over an `aw × ah` plane, clamped to it,
/// rounded half up.
pub fn bilinear(p: &[u16], aw: usize, ah: usize, x: f64, y: f64) -> u16 {
    let (mx, my) = ((aw - 1) as f64, (ah - 1) as f64);
    let x = x.clamp(0.0, mx);
    let y = y.clamp(0.0, my);
    let (x0, y0) = (x.floor(), y.floor());
    let (x1, y1) = ((x0 + 1.0).min(mx), (y0 + 1.0).min(my));
    let (tx, ty) = (x - x0, y - y0);
    let at = |xi: f64, yi: f64| f64::from(p[yi as usize * aw + xi as usize]);
    let v0 = at(x0, y0) * (1.0 - tx) + at(x1, y0) * tx;
    let v1 = at(x0, y1) * (1.0 - tx) + at(x1, y1) * tx;
    ((v0 * (1.0 - ty) + v1 * ty) + 0.5)
        .floor()
        .clamp(0.0, 65535.0) as u16
}

/// What the reference resamples.
pub struct Rule {
    /// Green's `kr`.
    pub green: [f64; 4],
    /// Red's `kr` when red is resampled.
    pub red: Option<[f64; 4]>,
    /// Blue's `kr` when blue is resampled.
    pub blue: Option<[f64; 4]>,
    /// The shared optical centre, normalised.
    pub cx: f64,
    pub cy: f64,
}

/// The CA step on an interleaved camera-RGB plane `width` wide, over `active`
/// (`top, left, bottom, right`, plane coordinates). Pixels outside it, and
/// green, pass through.
pub fn reference(cam: &[u16], width: usize, active: (u32, u32, u32, u32), rule: &Rule) -> Vec<u16> {
    let mut out = cam.to_vec();
    resample_in_place(cam, &mut out, width, active, rule);
    out
}

/// [`reference`] writing into `out` (already a copy of `cam`), so a 60 MP
/// caller holds two planes, not three.
pub fn resample_in_place(
    cam: &[u16],
    out: &mut [u16],
    width: usize,
    active: (u32, u32, u32, u32),
    rule: &Rule,
) {
    let (top, left, bottom, right) = active;
    let (top, left) = (top as usize, left as usize);
    let (aw, ah) = ((right as usize) - left, (bottom as usize) - top);
    let cx = rule.cx * (aw as f64 - 1.0);
    let cy = rule.cy * (ah as f64 - 1.0);
    let mx = cx.abs().max((aw as f64 - 1.0 - cx).abs());
    let my = cy.abs().max((ah as f64 - 1.0 - cy).abs());
    let m = mx.hypot(my);
    for (ch, set) in [(0usize, rule.red), (2, rule.blue)] {
        let Some(set) = set else { continue };
        let t = table(set, rule.green);
        let plane: Vec<u16> = (0..ah)
            .flat_map(|y| (0..aw).map(move |x| (x, y)))
            .map(|(x, y)| cam[((y + top) * width + x + left) * 3 + ch])
            .collect();
        for y in 0..ah {
            for x in 0..aw {
                let (fx, fy) = (x as f64, y as f64);
                let dx = (fx - cx) / m;
                let dy = (fy - cy) / m;
                let sigma = dx * dx + dy * dy;
                let u = (sigma * T as f64).min(T as f64);
                let j = (u.floor() as usize).min(T - 1);
                let k = t[j] + (t[j + 1] - t[j]) * (u - j as f64);
                let v = bilinear(&plane, aw, ah, cx + k * (fx - cx), cy + k * (fy - cy));
                out[((y + top) * width + x + left) * 3 + ch] = v;
            }
        }
    }
}
