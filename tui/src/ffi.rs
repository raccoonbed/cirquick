use std::ffi::c_double;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub real:  c_double,
    pub imag:  c_double,
    pub mag:   c_double,
    pub phase: c_double,
}

extern "C" {
    pub fn from_rect(real: c_double, imag: c_double) -> Complex;
    pub fn from_polar(mag: c_double, phase_deg: c_double) -> Complex;

    pub fn ohm_voltage(i: Complex, z: Complex) -> Complex;
    pub fn ohm_current(v: Complex, z: Complex) -> Complex;

    pub fn voltage_divider(v_s: Complex, z1: Complex, z2: Complex) -> Complex;
    pub fn current_divider(i_s: Complex, z1: Complex, z2: Complex) -> Complex;

    pub fn complex_power(v: Complex, i: Complex) -> Complex;
    pub fn power_from_IZ(i: Complex, z: Complex) -> Complex;
    pub fn power_from_VZ(v: Complex, z: Complex) -> Complex;

    pub fn Z_R(r: c_double) -> Complex;
    pub fn Z_L(l: c_double, freq: c_double) -> Complex;
    pub fn Z_C(c: c_double, freq: c_double) -> Complex;
    pub fn Z_series2(a: Complex, b: Complex) -> Complex;
    pub fn Z_parallel2(a: Complex, b: Complex) -> Complex;

    pub fn to_rms(peak: Complex) -> Complex;
    pub fn to_peak(rms: Complex) -> Complex;

    pub fn pmax_compute(vth: Complex, zth: Complex, rms: std::ffi::c_int) -> c_double;
    pub fn power_triangle_type(s: Complex) -> std::ffi::c_int;
    pub fn solve_pt(mode: std::ffi::c_int, x: c_double, y: c_double) -> PowerTriangle;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct PowerTriangle {
    pub P:     f64,
    pub Q:     f64,
    pub S:     f64,
    pub theta: f64,
    pub pf:    f64,
}

pub type TriangleInput = std::ffi::c_int;
pub const PT_P_Q:     TriangleInput = 0;
pub const PT_P_S:     TriangleInput = 1;
pub const PT_P_THETA: TriangleInput = 2;
pub const PT_P_PF:    TriangleInput = 3;
pub const PT_Q_S:     TriangleInput = 4;
pub const PT_Q_THETA: TriangleInput = 5;
pub const PT_Q_PF:    TriangleInput = 6;
pub const PT_S_THETA: TriangleInput = 7;
pub const PT_S_PF:    TriangleInput = 8;

/// Convert a solved PowerTriangle to a Complex for the triangle display.
/// theta from C is in radians; Complex.phase is stored in degrees.
pub fn pt_to_complex(pt: PowerTriangle) -> Complex {
    Complex { real: pt.P, imag: pt.Q, mag: pt.S, phase: pt.theta.to_degrees() }
}

// ---------------------------------------------------------------------------
// Safe constructors
// ---------------------------------------------------------------------------

pub fn rect(real: f64, imag: f64) -> Complex {
    unsafe { from_rect(real, imag) }
}

pub fn polar(mag: f64, phase: f64) -> Complex {
    unsafe { from_polar(mag, phase) }
}

// Parse the 3 answers that represent one complex input in the form:
//   answers[i]   = "1" (rect) or "2" (polar)
//   answers[i+1] = first value  (real or magnitude)
//   answers[i+2] = second value (imag or phase)
pub fn parse_complex(answers: &[String], i: usize) -> Complex {
    let choice = answers[i].trim().parse::<usize>().unwrap_or(1);
    let v1 = answers[i + 1].trim().parse::<f64>().unwrap_or(0.0);
    let v2 = answers[i + 2].trim().parse::<f64>().unwrap_or(0.0);
    if choice == 2 { polar(v1, v2) } else { rect(v1, v2) }
}

// ---------------------------------------------------------------------------
// Display helpers (ratatui owns stdout, so never call C printf functions)
// ---------------------------------------------------------------------------

pub fn fmt_complex(z: Complex) -> String {
    let sign = if z.imag >= 0.0 { '+' } else { '-' };
    format!(
        "{:.4}  {}  j{:.4}\n= {:.4} ∠ {:.4}°",
        z.real, sign, z.imag.abs(), z.mag, z.phase
    )
}

pub fn fmt_power(s: Complex) -> String {
    let pf = if s.mag > 1e-12 { s.real / s.mag } else { 0.0 };
    let pf_type = if s.imag > 1e-9 {
        "lagging  (inductive)"
    } else if s.imag < -1e-9 {
        "leading  (capacitive)"
    } else {
        "unity"
    };
    format!(
        "S  =  {:.4} VA ∠ {:.4}°\nP  =  {:.4} W\nQ  =  {:.4} VAR\npf =  {:.4}  ({})",
        s.mag, s.phase, s.real, s.imag, pf, pf_type
    )
}
