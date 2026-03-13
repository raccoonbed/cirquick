use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CLine},
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame,
};

use crate::app::{App, Mode};
use crate::ffi;
use crate::forms::StepKind;

const CYAN: Color = Color::Cyan;
const YELLOW: Color = Color::Yellow;
const GREEN: Color = Color::Green;
const WHITE: Color = Color::White;
const GRAY: Color = Color::DarkGray;

fn bold(c: Color) -> Style {
    Style::default().fg(c).add_modifier(Modifier::BOLD)
}
fn dim() -> Style {
    Style::default().fg(GRAY)
}

fn outer_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(bold(CYAN))
}

fn centered(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(v[1])[1]
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn draw(frame: &mut Frame, app: &App) {
    match app.mode {
        Mode::Menu => draw_menu(frame, app),
        Mode::Form => draw_form(frame, app),
        Mode::Result => draw_result(frame, app),
        Mode::Triangle  => draw_triangle(frame, app),
        Mode::Solver    => draw_solver(frame, app),
        Mode::MapSolver => draw_map_solver(frame, app),
        Mode::MapResult => draw_map_result(frame, app),
        Mode::Quit      => {}
    }
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

fn draw_menu(frame: &mut Frame, app: &App) {
    let area = centered(55, 85, frame.area());
    let block = outer_block("ECE 211  —  Circuit Analysis");

    let items: Vec<ListItem> = app
        .forms
        .iter()
        .enumerate()
        .map(|(i, form)| {
            let num = Span::styled(format!(" {:>2}.  ", i + 1), dim());
            let name = Span::styled(form.title, Style::default().fg(WHITE));
            let line = Line::from(vec![num, name]);
            if i == app.selected {
                ListItem::new(line).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(CYAN)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let footer = Line::from(vec![
        Span::styled("  ↑↓ / j·k", dim()),
        Span::styled("  navigate    ", dim()),
        Span::styled("Enter", bold(CYAN)),
        Span::styled("  select    ", dim()),
        Span::styled("q", bold(CYAN)),
        Span::styled("  quit  ", dim()),
    ]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    frame.render_stateful_widget(List::new(items).block(block), chunks[0], &mut state);
    frame.render_widget(
        Paragraph::new(footer).alignment(Alignment::Center),
        chunks[1],
    );
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

fn draw_form(frame: &mut Frame, app: &App) {
    let Some(runner) = app.runner.as_ref() else {
        return;
    };
    let form = &runner.form;

    let area = centered(65, 90, frame.area());
    let block = outer_block(form.title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(inner);

    let done: Vec<ListItem> = form.steps[..runner.step]
        .iter()
        .zip(runner.answers.iter())
        .map(|(step, ans)| {
            let lbl = Span::styled(format!("  {}  ", step.label), dim());
            let val = Span::styled(format!("→  {}", ans), bold(GREEN));
            ListItem::new(Line::from(vec![lbl, val]))
        })
        .collect();
    frame.render_widget(List::new(done), chunks[0]);

    if runner.step < form.steps.len() {
        let step = &form.steps[runner.step];

        let sep = Paragraph::new(Line::from(vec![
            Span::styled("─── ", dim()),
            Span::styled(&step.label, bold(YELLOW)),
            Span::styled(" ───", dim()),
        ]));
        frame.render_widget(sep, chunks[1]);

        let prompt_text = match &step.kind {
            StepKind::Float => format!("  > {}_", runner.input_buf),
            StepKind::YesNo => format!("  [y/n]  > {}_", runner.input_buf),
            StepKind::Choice(opts) => {
                let mut s = String::new();
                for (i, o) in opts.iter().enumerate() {
                    s.push_str(&format!("  {}. {}\n", i + 1, o));
                }
                s.push_str(&format!("  > {}_", runner.input_buf));
                s
            }
        };

        let prompt = Paragraph::new(prompt_text)
            .style(bold(WHITE))
            .wrap(Wrap { trim: false });
        frame.render_widget(prompt, chunks[2]);
    }

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  Enter", bold(CYAN)),
        Span::styled("  confirm    ", dim()),
        Span::styled("Esc", bold(CYAN)),
        Span::styled("  back to menu  ", dim()),
    ]));
    frame.render_widget(hint, chunks[3]);
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

fn draw_result(frame: &mut Frame, app: &App) {
    let area = centered(60, 70, frame.area());
    let block = outer_block("Result");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let result = Paragraph::new(app.result.as_str())
        .style(bold(GREEN))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(result, chunks[0]);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  any key", bold(CYAN)),
        Span::styled("  return to menu  ", dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[1]);
}

// ---------------------------------------------------------------------------
// Power Triangle Solver
// ---------------------------------------------------------------------------

fn draw_solver(frame: &mut Frame, app: &App) {
    let area = centered(50, 90, frame.area());
    let block = outer_block("Power Triangle Solver");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // instruction
            Constraint::Length(3), // P
            Constraint::Length(3), // Q
            Constraint::Length(3), // S
            Constraint::Length(3), // θ
            Constraint::Length(3), // pf
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // hint
        ])
        .split(inner);

    let instr = Paragraph::new(
        "  Fill in any 2 fields, then press Enter to solve.\n  Leading triangle: enter negative pf, Q, or θ."
    ).style(bold(YELLOW));
    frame.render_widget(instr, chunks[0]);

    let labels = ["P  (W)", "Q  (VAR)", "S  (VA)", "θ  (°)", "pf"];
    for (i, label) in labels.iter().enumerate() {
        let focused = i == app.solver_focus;
        let border_style = if focused {
            bold(CYAN)
        } else {
            Style::default().fg(GRAY)
        };
        let box_block = Block::default()
            .title(format!(" {} ", label))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);
        let box_inner = box_block.inner(chunks[i + 1]);
        frame.render_widget(box_block, chunks[i + 1]);
        let cursor = if focused { "_" } else { "" };
        let p = Paragraph::new(format!("  {}{}", app.solver_fields[i], cursor)).style(if focused {
            bold(WHITE)
        } else {
            dim()
        });
        frame.render_widget(p, box_inner);
    }

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  Tab/↑↓", bold(CYAN)),
        Span::styled("  navigate    ", dim()),
        Span::styled("Enter", bold(CYAN)),
        Span::styled("  solve    ", dim()),
        Span::styled("Esc", bold(CYAN)),
        Span::styled("  back  ", dim()),
    ]));
    frame.render_widget(hint, chunks[7]);
}

// ---------------------------------------------------------------------------
// Power Triangle
// ---------------------------------------------------------------------------

fn draw_triangle(frame: &mut Frame, app: &App) {
    let s = app.triangle_s;
    let kind = unsafe { ffi::power_triangle_type(s) };

    let pf = if s.mag > 1e-12 { s.real / s.mag } else { 0.0 };
    let pf_str = if kind > 0 {
        "lagging  (inductive)"
    } else if kind < 0 {
        "leading  (capacitive)"
    } else {
        "unity"
    };

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Canvas ─────────────────────────────────────────────────────────────
    let canvas_block = outer_block("Power Triangle");

    // Scale P and Q to fit canvas (hypotenuse maps to 65 units)
    let scale = if s.mag > 1e-12 { 65.0 / s.mag } else { 1.0 };
    let p_s = s.real.abs() * scale;
    let q_s = s.imag.abs() * scale;
    let ang = s.phase;

    // Lagging: right angle at bottom-right
    //   A (bottom-left, θ) — B (bottom-right, 90°) — C (top-right)
    // Leading: right angle at top-right
    //   A (top-left, θ)    — B (top-right, 90°)    — C (bottom-right)
    let (ax, ay, bx, by, cx, cy, p_lx, p_ly, q_lx, q_ly, s_lx, s_ly, theta_lx, theta_ly) =
        if kind >= 0 {
            // Lagging
            let ax = 15.0;
            let ay = 15.0;
            let bx = ax + p_s;
            let by = ay;
            let cx = bx;
            let cy = ay + q_s;
            (
                ax,
                ay,
                bx,
                by,
                cx,
                cy,
                ax + p_s / 2.0,
                ay - 5.0, // P label below base
                bx + 3.0,
                by + q_s / 2.0, // Q label right of side
                ax + p_s / 2.0 - 8.0,
                ay + q_s / 2.0 + 3.0, // |S| label on hyp
                ax + 8.0,
                ay + 4.0,
            ) // θ label near A
        } else {
            // Leading
            let ax = 15.0;
            let ay = 85.0;
            let bx = ax + p_s;
            let by = ay;
            let cx = bx;
            let cy = ay - q_s;
            (
                ax,
                ay,
                bx,
                by,
                cx,
                cy,
                ax + p_s / 2.0,
                ay + 4.0, // P label above base
                bx + 3.0,
                by - q_s / 2.0, // Q label right of side
                ax + p_s / 2.0 - 8.0,
                ay - q_s / 2.0, // |S| label on hyp
                ax + 8.0,
                ay - 6.0,
            ) // θ label near A (between P and |S|)
        };

    let canvas = Canvas::default()
        .block(canvas_block)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(move |ctx| {
            // P — base/top (yellow)
            ctx.draw(&CLine {
                x1: ax,
                y1: ay,
                x2: bx,
                y2: by,
                color: YELLOW,
            });
            // Q — vertical side (cyan)
            ctx.draw(&CLine {
                x1: bx,
                y1: by,
                x2: cx,
                y2: cy,
                color: CYAN,
            });
            // |S| — hypotenuse (green)
            ctx.draw(&CLine {
                x1: ax,
                y1: ay,
                x2: cx,
                y2: cy,
                color: GREEN,
            });

            // Labels
            ctx.print(
                p_lx,
                p_ly,
                Line::styled(format!("P={:.2}W", s.real.abs()), bold(YELLOW)),
            );
            ctx.print(
                q_lx,
                q_ly,
                Line::styled(format!("Q={:.2}VAR", s.imag.abs()), bold(CYAN)),
            );
            ctx.print(
                s_lx,
                s_ly,
                Line::styled(format!("|S|={:.2}VA", s.mag), bold(GREEN)),
            );
            ctx.print(
                theta_lx,
                theta_ly,
                Line::styled(format!("θ={:.1}°", ang), bold(WHITE)),
            );
        });

    frame.render_widget(canvas, chunks[0]);

    // ── Info panel ─────────────────────────────────────────────────────────
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(bold(CYAN));
    let info_inner = info_block.inner(chunks[1]);
    frame.render_widget(info_block, chunks[1]);

    let info = vec![
        Line::from(vec![
            Span::styled("  P  = ", bold(YELLOW)),
            Span::styled(format!("{:.4} W", s.real), bold(WHITE)),
            Span::styled("    Q  = ", bold(CYAN)),
            Span::styled(format!("{:.4} VAR", s.imag), bold(WHITE)),
            Span::styled("    |S| = ", bold(GREEN)),
            Span::styled(format!("{:.4} VA", s.mag), bold(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  pf = ", bold(YELLOW)),
            Span::styled(format!("{:.4}", pf), bold(WHITE)),
            Span::styled("    θ  = ", bold(CYAN)),
            Span::styled(format!("{:.4}°", s.phase), bold(WHITE)),
            Span::styled(format!("    {}", pf_str), dim()),
        ]),
    ];
    frame.render_widget(Paragraph::new(info), info_inner);

    // ── Hint ───────────────────────────────────────────────────────────────
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  any key", bold(CYAN)),
        Span::styled("  return to menu  ", dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[2]);
}

// ---------------------------------------------------------------------------
// Max Average Power Solver
// ---------------------------------------------------------------------------

fn draw_map_solver(frame: &mut Frame, app: &App) {
    let area = centered(62, 92, frame.area());
    let block = outer_block("Max Average Power Transfer");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // instruction
            Constraint::Length(3), // mode selector
            Constraint::Length(1), // separator
            Constraint::Length(3), // x group label + fields
            Constraint::Length(1), // separator
            Constraint::Length(3), // Zth group fields
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // hint
        ])
        .split(inner);

    let mode_names = ["Vth  (peak)", "In  (peak)", "Vth  (RMS)", "In  (RMS)"];
    let instr = Paragraph::new(
        "  Fill in the active source (Vth or In) and Zth, then press Enter.\n  Press 1-4 to select mode."
    ).style(bold(YELLOW));
    frame.render_widget(instr, chunks[0]);

    // Mode selector row
    let mode_block = Block::default()
        .title(" Mode ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.map_focus == 0 { bold(CYAN) } else { dim() });
    let mode_inner = mode_block.inner(chunks[1]);
    frame.render_widget(mode_block, chunks[1]);
    let mode_spans: Vec<Span> = mode_names.iter().enumerate().map(|(i, name)| {
        if i == app.map_mode {
            Span::styled(format!("  [{}] {} ", i + 1, name), bold(CYAN))
        } else {
            Span::styled(format!("  {}. {} ", i + 1, name), dim())
        }
    }).collect();
    frame.render_widget(
        Paragraph::new(Line::from(mode_spans)),
        mode_inner,
    );

    // x group (Vth or In depending on mode)
    let x_label = match app.map_mode {
        0 => "Vth  (peak)",
        1 => "In  (peak)",
        2 => "Vth  (RMS)",
        _ => "In  (RMS)",
    };
    let x_color = match app.map_mode { 0 | 2 => GREEN, _ => YELLOW };
    let sep1 = Paragraph::new(Line::from(vec![
        Span::styled("─── ", dim()),
        Span::styled(x_label, bold(x_color)),
        Span::styled(" ───", dim()),
    ]));
    frame.render_widget(sep1, chunks[2]);

    draw_complex_row(frame, chunks[3], app, 1, x_label);

    let sep2 = Paragraph::new(Line::from(vec![
        Span::styled("─── ", dim()),
        Span::styled("Zth  (Thevenin impedance)", bold(CYAN)),
        Span::styled(" ───", dim()),
    ]));
    frame.render_widget(sep2, chunks[4]);

    draw_complex_row(frame, chunks[5], app, 4, "Zth");

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  Tab/↑↓", bold(CYAN)),
        Span::styled("  navigate    ", dim()),
        Span::styled("1/2", bold(CYAN)),
        Span::styled("  rect/polar    ", dim()),
        Span::styled("Enter", bold(CYAN)),
        Span::styled("  solve    ", dim()),
        Span::styled("Esc", bold(CYAN)),
        Span::styled("  back  ", dim()),
    ]));
    frame.render_widget(hint, chunks[7]);
}

/// Renders a row of 3 boxes for one complex input starting at focus_base.
/// focus_base+0 = form selector, focus_base+1 = v1, focus_base+2 = v2
fn draw_complex_row(frame: &mut Frame, area: Rect, app: &App, focus_base: usize, _label: &str) {
    let fi = focus_base - 1; // index into map_fields
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
        ])
        .split(area);

    let form_val = app.map_fields[fi].trim().parse::<usize>().unwrap_or(1);
    let form_name = if form_val == 2 { "Polar" } else { "Rect" };
    let v1_label = if form_val == 2 { "mag" } else { "real" };
    let v2_label = if form_val == 2 { "phase (°)" } else { "imag" };

    let sub_data = [
        (focus_base,     format!(" {} ", form_name), fi),
        (focus_base + 1, format!(" {} ", v1_label),  fi + 1),
        (focus_base + 2, format!(" {} ", v2_label),  fi + 2),
    ];

    for (focus, title, field_idx) in sub_data {
        let focused = focus == app.map_focus;
        let border_style = if focused { bold(CYAN) } else { dim() };
        let b = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);
        let inner = b.inner(cols[focus - focus_base]);
        frame.render_widget(b, cols[focus - focus_base]);
        let cursor = if focused { "_" } else { "" };
        let text = format!("  {}{}", app.map_fields[field_idx], cursor);
        frame.render_widget(
            Paragraph::new(text).style(if focused { bold(WHITE) } else { dim() }),
            inner,
        );
    }
}

// ---------------------------------------------------------------------------
// Max Average Power Result
// ---------------------------------------------------------------------------

fn draw_map_result(frame: &mut Frame, app: &App) {
    let Some(map) = app.map_result else { return };
    let pmax = map.pmax;

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // canvas
            Constraint::Length(7), // dashboard
            Constraint::Length(1), // hint
        ])
        .split(area);

    // ── Circuit canvas ──────────────────────────────────────────────────────
    let canvas_block = outer_block("Thevenin  ║  Norton  (ZL = Zth*)");
    let canvas = Canvas::default()
        .block(canvas_block)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(move |ctx| {
            // ── Thevenin (left half: x 4-44) ────────────────────────────
            // Outer rectangle
            ctx.draw(&CLine { x1:  4.0, y1: 80.0, x2: 44.0, y2: 80.0, color: WHITE });
            ctx.draw(&CLine { x1:  4.0, y1: 80.0, x2:  4.0, y2: 25.0, color: WHITE });
            ctx.draw(&CLine { x1:  4.0, y1: 25.0, x2: 44.0, y2: 25.0, color: WHITE });
            ctx.draw(&CLine { x1: 44.0, y1: 25.0, x2: 44.0, y2: 80.0, color: WHITE });
            // Zth: break in top wire with coloured segment
            ctx.draw(&CLine { x1:  4.0, y1: 80.0, x2: 17.0, y2: 80.0, color: YELLOW });
            ctx.draw(&CLine { x1: 30.0, y1: 80.0, x2: 44.0, y2: 80.0, color: YELLOW });
            ctx.draw(&CLine { x1: 17.0, y1: 80.0, x2: 17.0, y2: 84.0, color: YELLOW });
            ctx.draw(&CLine { x1: 17.0, y1: 84.0, x2: 30.0, y2: 84.0, color: YELLOW });
            ctx.draw(&CLine { x1: 30.0, y1: 84.0, x2: 30.0, y2: 80.0, color: YELLOW });
            // Vth: left side with source symbol
            ctx.draw(&CLine { x1:  4.0, y1: 56.0, x2:  4.0, y2: 25.0, color: GREEN });
            ctx.draw(&CLine { x1:  4.0, y1: 80.0, x2:  4.0, y2: 62.0, color: GREEN });
            // source circle
            ctx.draw(&CLine { x1:  1.0, y1: 62.0, x2:  7.0, y2: 62.0, color: GREEN });
            ctx.draw(&CLine { x1:  1.0, y1: 56.0, x2:  7.0, y2: 56.0, color: GREEN });
            ctx.draw(&CLine { x1:  1.0, y1: 56.0, x2:  1.0, y2: 62.0, color: GREEN });
            ctx.draw(&CLine { x1:  7.0, y1: 56.0, x2:  7.0, y2: 62.0, color: GREEN });
            // Zl: right side
            ctx.draw(&CLine { x1: 44.0, y1: 80.0, x2: 44.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 41.0, y1: 62.0, x2: 47.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 41.0, y1: 56.0, x2: 47.0, y2: 56.0, color: CYAN });
            ctx.draw(&CLine { x1: 41.0, y1: 56.0, x2: 41.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 47.0, y1: 56.0, x2: 47.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 44.0, y1: 25.0, x2: 44.0, y2: 56.0, color: CYAN });
            // Labels
            ctx.print( 0.0, 91.0, Line::styled("THEVENIN", bold(WHITE)));
            ctx.print(18.0, 85.5, Line::styled("Zth", bold(YELLOW)));
            ctx.print( 0.0, 58.0, Line::styled("+", bold(GREEN)));
            ctx.print( 0.0, 56.0, Line::styled("Vth", bold(GREEN)));
            ctx.print( 0.0, 54.5, Line::styled("-", bold(GREEN)));
            ctx.print(45.0, 58.0, Line::styled("Zl", bold(CYAN)));
            ctx.print(40.0, 81.5, Line::styled("A", bold(WHITE)));
            ctx.print(40.0, 22.0, Line::styled("B", bold(WHITE)));

            // ── Divider ─────────────────────────────────────────────────
            ctx.draw(&CLine { x1: 50.0, y1: 10.0, x2: 50.0, y2: 90.0, color: GRAY });

            // ── Norton (right half: x 56-96) ────────────────────────────
            // Outer rectangle
            ctx.draw(&CLine { x1: 56.0, y1: 80.0, x2: 96.0, y2: 80.0, color: WHITE });
            ctx.draw(&CLine { x1: 56.0, y1: 80.0, x2: 56.0, y2: 25.0, color: WHITE });
            ctx.draw(&CLine { x1: 56.0, y1: 25.0, x2: 96.0, y2: 25.0, color: WHITE });
            ctx.draw(&CLine { x1: 96.0, y1: 25.0, x2: 96.0, y2: 80.0, color: WHITE });
            // In: left side with source symbol
            ctx.draw(&CLine { x1: 56.0, y1: 56.0, x2: 56.0, y2: 25.0, color: GREEN });
            ctx.draw(&CLine { x1: 56.0, y1: 80.0, x2: 56.0, y2: 62.0, color: GREEN });
            ctx.draw(&CLine { x1: 53.0, y1: 62.0, x2: 59.0, y2: 62.0, color: GREEN });
            ctx.draw(&CLine { x1: 53.0, y1: 56.0, x2: 59.0, y2: 56.0, color: GREEN });
            ctx.draw(&CLine { x1: 53.0, y1: 56.0, x2: 53.0, y2: 62.0, color: GREEN });
            ctx.draw(&CLine { x1: 59.0, y1: 56.0, x2: 59.0, y2: 62.0, color: GREEN });
            // Zth: middle divider
            ctx.draw(&CLine { x1: 76.0, y1: 80.0, x2: 76.0, y2: 62.0, color: YELLOW });
            ctx.draw(&CLine { x1: 73.0, y1: 62.0, x2: 79.0, y2: 62.0, color: YELLOW });
            ctx.draw(&CLine { x1: 73.0, y1: 56.0, x2: 79.0, y2: 56.0, color: YELLOW });
            ctx.draw(&CLine { x1: 73.0, y1: 56.0, x2: 73.0, y2: 62.0, color: YELLOW });
            ctx.draw(&CLine { x1: 79.0, y1: 56.0, x2: 79.0, y2: 62.0, color: YELLOW });
            ctx.draw(&CLine { x1: 76.0, y1: 25.0, x2: 76.0, y2: 56.0, color: YELLOW });
            // Zl: right side
            ctx.draw(&CLine { x1: 96.0, y1: 80.0, x2: 96.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 93.0, y1: 62.0, x2: 99.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 93.0, y1: 56.0, x2: 99.0, y2: 56.0, color: CYAN });
            ctx.draw(&CLine { x1: 93.0, y1: 56.0, x2: 93.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 99.0, y1: 56.0, x2: 99.0, y2: 62.0, color: CYAN });
            ctx.draw(&CLine { x1: 96.0, y1: 25.0, x2: 96.0, y2: 56.0, color: CYAN });
            // Labels
            ctx.print(56.0, 91.0, Line::styled("NORTON", bold(WHITE)));
            ctx.print(53.0, 58.0, Line::styled("In", bold(GREEN)));
            ctx.print(74.0, 58.0, Line::styled("Zth", bold(YELLOW)));
            ctx.print(97.0, 58.0, Line::styled("Zl", bold(CYAN)));
            ctx.print(92.0, 81.5, Line::styled("A", bold(WHITE)));
            ctx.print(92.0, 22.0, Line::styled("B", bold(WHITE)));
        });
    frame.render_widget(canvas, chunks[0]);

    // ── Dashboard ────────────────────────────────────────────────────────────
    let dash_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(bold(CYAN));
    let dash_inner = dash_block.inner(chunks[1]);
    frame.render_widget(dash_block, chunks[1]);

    let zl_sign = if map.Zl.imag >= 0.0 { '+' } else { '-' };
    let zth_sign = if map.Zth.imag >= 0.0 { '+' } else { '-' };
    let vth_sign = if map.Vth.imag >= 0.0 { '+' } else { '-' };
    let in_sign  = if map.In.imag  >= 0.0 { '+' } else { '-' };

    let dash = vec![
        Line::from(vec![
            Span::styled("  Vth = ", bold(GREEN)),
            Span::styled(
                format!("{:.4} {} j{:.4}  =  {:.4} ∠ {:.4}°",
                    map.Vth.real, vth_sign, map.Vth.imag.abs(), map.Vth.mag, map.Vth.phase),
                bold(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  In  = ", bold(GREEN)),
            Span::styled(
                format!("{:.4} {} j{:.4}  =  {:.4} ∠ {:.4}°",
                    map.In.real, in_sign, map.In.imag.abs(), map.In.mag, map.In.phase),
                bold(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  Zth = ", bold(YELLOW)),
            Span::styled(
                format!("{:.4} {} j{:.4}  =  {:.4} ∠ {:.4}°",
                    map.Zth.real, zth_sign, map.Zth.imag.abs(), map.Zth.mag, map.Zth.phase),
                bold(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  Zl  = ", bold(CYAN)),
            Span::styled(
                format!("{:.4} {} j{:.4}  =  {:.4} ∠ {:.4}°",
                    map.Zl.real, zl_sign, map.Zl.imag.abs(), map.Zl.mag, map.Zl.phase),
                bold(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  Pmax = ", bold(YELLOW)),
            Span::styled(format!("{:.4} W", pmax), bold(WHITE)),
        ]),
    ];
    frame.render_widget(Paragraph::new(dash), dash_inner);

    // ── Hint ─────────────────────────────────────────────────────────────────
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  any key", bold(CYAN)),
        Span::styled("  return to menu  ", dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[2]);
}
