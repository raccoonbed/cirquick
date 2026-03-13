use crate::ffi;

// ---------------------------------------------------------------------------
// Step — one question shown to the user
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Step {
    pub label:  String,
    pub kind:   StepKind,
}

#[derive(Clone)]
pub enum StepKind {
    Float,
    Choice(Vec<&'static str>),
    YesNo,
}

impl Step {
    fn float(label: impl Into<String>) -> Self {
        Self { label: label.into(), kind: StepKind::Float }
    }
    fn choice(label: impl Into<String>, opts: Vec<&'static str>) -> Self {
        Self { label: label.into(), kind: StepKind::Choice(opts) }
    }
    fn yesno(label: impl Into<String>) -> Self {
        Self { label: label.into(), kind: StepKind::YesNo }
    }
}

// Three consecutive steps that collect one Complex value.
fn complex(name: &str) -> Vec<Step> {
    vec![
        Step::choice(
            format!("{name}  [form]"),
            vec!["Rectangular  (real + j·imag)", "Polar  (mag ∠ phase°)"],
        ),
        Step::float(format!("{name}  —  real  /  magnitude")),
        Step::float(format!("{name}  —  imag  /  phase (°)")),
    ]
}

// ---------------------------------------------------------------------------
// Form definition
// ---------------------------------------------------------------------------

pub enum FormResult {
    Text(fn(&[String]) -> String),
    Triangle(fn(&[String]) -> ffi::Complex),
    Solver,
    MapSolver,
}

pub struct Form {
    pub title:  &'static str,
    pub steps:  Vec<Step>,
    pub result: FormResult,
}

// ---------------------------------------------------------------------------
// FormRunner — tracks progress through a form's steps
// ---------------------------------------------------------------------------

pub struct FormRunner {
    pub form:      Form,
    pub step:      usize,
    pub answers:   Vec<String>,
    pub input_buf: String,
}

impl FormRunner {
    pub fn new(form: Form) -> Self {
        Self { form, step: 0, answers: Vec::new(), input_buf: String::new() }
    }

    /// Called on Enter. Validates and records the answer.
    /// Returns true when all steps are complete.
    pub fn submit(&mut self) -> bool {
        let raw = self.input_buf.trim().to_string();

        let ok = match &self.form.steps[self.step].kind {
            StepKind::Float  => raw.parse::<f64>().is_ok(),
            StepKind::YesNo  => matches!(raw.to_lowercase().as_str(), "y" | "n"),
            StepKind::Choice(opts) => {
                raw.parse::<usize>().map_or(false, |n| n >= 1 && n <= opts.len())
            }
        };

        if !ok {
            self.input_buf.clear();
            return false;
        }

        self.answers.push(raw);
        self.input_buf.clear();
        self.step += 1;
        self.step >= self.form.steps.len()
    }
}

// ---------------------------------------------------------------------------
// All forms
// ---------------------------------------------------------------------------

pub fn all_forms() -> Vec<Form> {
    vec![
        // 1 ─ Max Power Transfer
        Form {
            title: "Max Power Transfer",
            steps: {
                let mut s = complex("Vth  (Thevenin voltage)");
                s.extend(complex("Zth  (Thevenin impedance)"));
                s.push(Step::yesno("Is Vth already RMS?  [y/n]"));
                s
            },
            result: FormResult::Text(|a| {
                let vth  = ffi::parse_complex(a, 0);
                let zth  = ffi::parse_complex(a, 3);
                let rms  = if a[6].trim().to_lowercase() == "y" { 1 } else { 0 };
                let pmax = unsafe { ffi::pmax_compute(vth, zth, rms) };
                format!(
                    "Vth  =  {}\nZth  =  {}\n\nZL = Zth* = {:.4} - j{:.4} Ω\n\nP_max = {:.4} W",
                    ffi::fmt_complex(vth), ffi::fmt_complex(zth),
                    zth.real, zth.imag.abs(), pmax
                )
            }),
        },

        // 2 ─ Ohm's Law — Voltage
        Form {
            title: "Ohm's Law  —  Voltage  (V = I·Z)",
            steps: {
                let mut s = complex("I  (current)");
                s.extend(complex("Z  (impedance)"));
                s
            },
            result: FormResult::Text(|a| {
                let i = ffi::parse_complex(a, 0);
                let z = ffi::parse_complex(a, 3);
                let v = unsafe { ffi::ohm_voltage(i, z) };
                format!("V = I·Z\n\n{}", ffi::fmt_complex(v))
            }),
        },

        // 3 ─ Ohm's Law — Current
        Form {
            title: "Ohm's Law  —  Current  (I = V/Z)",
            steps: {
                let mut s = complex("V  (voltage)");
                s.extend(complex("Z  (impedance)"));
                s
            },
            result: FormResult::Text(|a| {
                let v = ffi::parse_complex(a, 0);
                let z = ffi::parse_complex(a, 3);
                let i = unsafe { ffi::ohm_current(v, z) };
                format!("I = V/Z\n\n{}", ffi::fmt_complex(i))
            }),
        },

        // 4 ─ Voltage Divider
        Form {
            title: "Voltage Divider  —  V across Z2",
            steps: {
                let mut s = complex("Vs  (source voltage)");
                s.extend(complex("Z1"));
                s.extend(complex("Z2"));
                s
            },
            result: FormResult::Text(|a| {
                let vs = ffi::parse_complex(a, 0);
                let z1 = ffi::parse_complex(a, 3);
                let z2 = ffi::parse_complex(a, 6);
                let v2 = unsafe { ffi::voltage_divider(vs, z1, z2) };
                format!("V_Z2 = Vs · Z2/(Z1+Z2)\n\n{}", ffi::fmt_complex(v2))
            }),
        },

        // 5 ─ Current Divider
        Form {
            title: "Current Divider  —  I through Z2",
            steps: {
                let mut s = complex("Is  (source current)");
                s.extend(complex("Z1"));
                s.extend(complex("Z2"));
                s
            },
            result: FormResult::Text(|a| {
                let is = ffi::parse_complex(a, 0);
                let z1 = ffi::parse_complex(a, 3);
                let z2 = ffi::parse_complex(a, 6);
                let i2 = unsafe { ffi::current_divider(is, z1, z2) };
                format!("I_Z2 = Is · Z1/(Z1+Z2)\n\n{}", ffi::fmt_complex(i2))
            }),
        },

        // 6 ─ Complex Power (V · I*)
        Form {
            title: "Complex Power  —  S = V·I*",
            steps: {
                let mut s = complex("V  (voltage, RMS)");
                s.extend(complex("I  (current, RMS)"));
                s
            },
            result: FormResult::Text(|a| {
                let v = ffi::parse_complex(a, 0);
                let i = ffi::parse_complex(a, 3);
                let s = unsafe { ffi::complex_power(v, i) };
                ffi::fmt_power(s)
            }),
        },

        // 7 ─ Complex Power (|I|² · Z)
        Form {
            title: "Complex Power  —  S = |I|²·Z",
            steps: {
                let mut s = complex("I  (current, RMS)");
                s.extend(complex("Z  (impedance)"));
                s
            },
            result: FormResult::Text(|a| {
                let i = ffi::parse_complex(a, 0);
                let z = ffi::parse_complex(a, 3);
                let s = unsafe { ffi::power_from_IZ(i, z) };
                ffi::fmt_power(s)
            }),
        },

        // 8 ─ Complex Power (|V|² / Z*)
        Form {
            title: "Complex Power  —  S = |V|²/Z*",
            steps: {
                let mut s = complex("V  (voltage, RMS)");
                s.extend(complex("Z  (impedance)"));
                s
            },
            result: FormResult::Text(|a| {
                let v = ffi::parse_complex(a, 0);
                let z = ffi::parse_complex(a, 3);
                let s = unsafe { ffi::power_from_VZ(v, z) };
                ffi::fmt_power(s)
            }),
        },

        // 9 ─ Impedance — Resistor
        Form {
            title: "Impedance  —  Resistor  Z_R",
            steps: vec![Step::float("R  (Ω)")],
            result: FormResult::Text(|a| {
                let r = a[0].trim().parse::<f64>().unwrap_or(0.0);
                let z = unsafe { ffi::Z_R(r) };
                format!("Z_R = R\n\n{}", ffi::fmt_complex(z))
            }),
        },

        // 10 ─ Impedance — Inductor
        Form {
            title: "Impedance  —  Inductor  Z_L",
            steps: vec![Step::float("L  (H)"), Step::float("freq  (Hz)")],
            result: FormResult::Text(|a| {
                let l = a[0].trim().parse::<f64>().unwrap_or(0.0);
                let f = a[1].trim().parse::<f64>().unwrap_or(0.0);
                let z = unsafe { ffi::Z_L(l, f) };
                format!("Z_L = jωL\n\n{}", ffi::fmt_complex(z))
            }),
        },

        // 11 ─ Impedance — Capacitor
        Form {
            title: "Impedance  —  Capacitor  Z_C",
            steps: vec![Step::float("C  (F)"), Step::float("freq  (Hz)")],
            result: FormResult::Text(|a| {
                let c = a[0].trim().parse::<f64>().unwrap_or(0.0);
                let f = a[1].trim().parse::<f64>().unwrap_or(0.0);
                let z = unsafe { ffi::Z_C(c, f) };
                format!("Z_C = 1/(jωC)\n\n{}", ffi::fmt_complex(z))
            }),
        },

        // 12 ─ Impedance — Series
        Form {
            title: "Impedance  —  Series  Za + Zb",
            steps: {
                let mut s = complex("Za");
                s.extend(complex("Zb"));
                s
            },
            result: FormResult::Text(|a| {
                let za = ffi::parse_complex(a, 0);
                let zb = ffi::parse_complex(a, 3);
                let z  = unsafe { ffi::Z_series2(za, zb) };
                format!("Z = Za + Zb\n\n{}", ffi::fmt_complex(z))
            }),
        },

        // 13 ─ Impedance — Parallel
        Form {
            title: "Impedance  —  Parallel  Za ∥ Zb",
            steps: {
                let mut s = complex("Za");
                s.extend(complex("Zb"));
                s
            },
            result: FormResult::Text(|a| {
                let za = ffi::parse_complex(a, 0);
                let zb = ffi::parse_complex(a, 3);
                let z  = unsafe { ffi::Z_parallel2(za, zb) };
                format!("Z = Za·Zb / (Za+Zb)\n\n{}", ffi::fmt_complex(z))
            }),
        },

        // 14 ─ Peak → RMS
        Form {
            title: "RMS / Peak  —  Peak → RMS  (÷ √2)",
            steps: complex("V_peak"),
            result: FormResult::Text(|a| {
                let v = ffi::parse_complex(a, 0);
                let r = unsafe { ffi::to_rms(v) };
                format!("V_rms = V_peak / √2\n\n{}", ffi::fmt_complex(r))
            }),
        },

        // 15 ─ RMS → Peak
        Form {
            title: "RMS / Peak  —  RMS → Peak  (× √2)",
            steps: complex("V_rms"),
            result: FormResult::Text(|a| {
                let v = ffi::parse_complex(a, 0);
                let p = unsafe { ffi::to_peak(v) };
                format!("V_peak = V_rms × √2\n\n{}", ffi::fmt_complex(p))
            }),
        },

        // 16 ─ Power Triangle
        Form {
            title: "Power Triangle",
            steps: complex("S  (complex power)"),
            result: FormResult::Triangle(|a| ffi::parse_complex(a, 0)),
        },

        // 17 ─ Power Triangle Solver
        Form {
            title: "Power Triangle Solver",
            steps: vec![],
            result: FormResult::Solver,
        },

        // 18 ─ Max Average Power Transfer
        Form {
            title: "Max Average Power Transfer",
            steps: vec![],
            result: FormResult::MapSolver,
        },
    ]
}
