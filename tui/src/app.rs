use crossterm::event::KeyCode;
use crate::ffi::{self, Complex};
use crate::forms::{all_forms, Form, FormResult, FormRunner};

pub enum Mode { Menu, Form, Result, Triangle, Solver, Quit }

pub struct App {
    pub mode:       Mode,
    // menu
    pub forms:      Vec<Form>,
    pub selected:   usize,
    // form
    pub runner:     Option<FormRunner>,
    // text result
    pub result:     String,
    // triangle result
    pub triangle_s: Complex,
    // solver
    pub solver_fields: [String; 5],
    pub solver_focus:  usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode:       Mode::Menu,
            forms:      all_forms(),
            selected:   0,
            runner:     None,
            result:     String::new(),
            triangle_s:    Complex { real: 0.0, imag: 0.0, mag: 0.0, phase: 0.0 },
            solver_fields: Default::default(),
            solver_focus:  0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            Mode::Menu     => self.menu_key(key),
            Mode::Form     => self.form_key(key),
            Mode::Result   => self.mode = Mode::Menu,
            Mode::Triangle => self.mode = Mode::Menu,
            Mode::Solver   => self.solver_key(key),
            Mode::Quit     => {}
        }
    }

    fn menu_key(&mut self, key: KeyCode) {
        let len = self.forms.len();
        match key {
            KeyCode::Up   | KeyCode::Char('k') => {
                if self.selected > 0 { self.selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < len - 1 { self.selected += 1; }
            }
            KeyCode::Enter => {
                let placeholder = Form {
                    title: "",
                    steps: vec![],
                    result: FormResult::Text(|_| String::new()),
                };
                let form = std::mem::replace(&mut self.forms[self.selected], placeholder);
                if matches!(form.result, FormResult::Solver) {
                    self.forms[self.selected] = form;
                    self.mode = Mode::Solver;
                } else {
                    self.runner = Some(FormRunner::new(form));
                    self.mode = Mode::Form;
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            _ => {}
        }
    }

    fn solver_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                for f in &mut self.solver_fields { f.clear(); }
                self.solver_focus = 0;
                self.mode = Mode::Menu;
            }
            KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                self.solver_focus = (self.solver_focus + 1) % 5;
            }
            KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                self.solver_focus = (self.solver_focus + 4) % 5;
            }
            KeyCode::Backspace => { self.solver_fields[self.solver_focus].pop(); }
            KeyCode::Enter => self.solver_submit(),
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                self.solver_fields[self.solver_focus].push(c);
            }
            _ => {}
        }
    }

    fn solver_submit(&mut self) {
        // Parse non-empty fields: indices 0=P, 1=Q, 2=S, 3=θ°, 4=pf
        let vals: Vec<Option<f64>> = self.solver_fields.iter()
            .map(|s| { let t = s.trim(); if t.is_empty() { None } else { t.parse().ok() } })
            .collect();

        let filled: Vec<usize> = vals.iter().enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(i, _)| i)
            .collect();

        if filled.len() != 2 { return; }

        let (ia, ib) = (filled[0], filled[1]);
        let a = vals[ia].unwrap();
        let b = vals[ib].unwrap();

        // For pf cases: acos() always returns positive, so we can't distinguish
        // lagging from leading without a sign. Convention: negative pf = leading,
        // and for (Q, pf): negative Q = leading.
        // For θ and Q cases, the sign is already encoded in the value.
        let leading = match (ia, ib) {
            (1, 4) => a < 0.0 || b < 0.0, // Q<0 or pf<0
            (_, 4) => b < 0.0,             // pf<0
            _ => false,
        };

        // Pass absolute values to C for pf cases; C always solves lagging.
        let (x, y) = if ib == 4 { (a.abs(), b.abs()) } else { (a, b) };

        let (mode, fx, fy): (ffi::TriangleInput, f64, f64) = match (ia, ib) {
            (0, 1) => (ffi::PT_P_Q,     x,  y),
            (0, 2) => (ffi::PT_P_S,     x,  y),
            (0, 3) => (ffi::PT_P_THETA, x,  y.to_radians()),
            (0, 4) => (ffi::PT_P_PF,    x,  y),
            (1, 2) => (ffi::PT_Q_S,     x,  y),
            (1, 3) => (ffi::PT_Q_THETA, x,  y.to_radians()),
            (1, 4) => (ffi::PT_Q_PF,    x,  y),
            (2, 3) => (ffi::PT_S_THETA, x,  y.to_radians()),
            (2, 4) => (ffi::PT_S_PF,    x,  y),
            _      => return,
        };

        let mut pt = unsafe { ffi::solve_pt(mode, fx, fy) };

        // Flip Q and θ for leading triangles derived from pf input
        if leading {
            pt.Q     = -pt.Q.abs();
            pt.theta = -pt.theta.abs();
        }

        self.triangle_s = ffi::pt_to_complex(pt);
        for f in &mut self.solver_fields { f.clear(); }
        self.solver_focus = 0;
        self.mode = Mode::Triangle;
    }

    fn form_key(&mut self, key: KeyCode) {
        let Some(runner) = self.runner.as_mut() else { return };
        match key {
            KeyCode::Esc => {
                let runner = self.runner.take().unwrap();
                self.forms[self.selected] = runner.form;
                self.mode = Mode::Menu;
            }
            KeyCode::Backspace => { runner.input_buf.pop(); }
            KeyCode::Enter => {
                if runner.submit() {
                    let runner = self.runner.take().unwrap();
                    match &runner.form.result {
                        FormResult::Text(f) => {
                            self.result = f(&runner.answers);
                            self.forms[self.selected] = runner.form;
                            self.mode = Mode::Result;
                        }
                        FormResult::Triangle(f) => {
                            self.triangle_s = f(&runner.answers);
                            self.forms[self.selected] = runner.form;
                            self.mode = Mode::Triangle;
                        }
                        FormResult::Solver => {} // handled in menu_key
                    }
                }
            }
            KeyCode::Char(c) => { runner.input_buf.push(c); }
            _ => {}
        }
    }
}
