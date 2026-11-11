# Cirquick

A small AC circuit analysis toolkit I wrote to check my circuits homework. The core is a C library that does phasor math and solves the problems I kept running into: impedance networks, dividers, complex power, power triangles, and maximum power transfer. A terminal UI sits on top of it so I can punch in numbers quickly.

> **Note:** The C solver library (`circuit.c`, `analysis.c`) is written entirely by hand. The terminal UI in `tui/` was built with AI assistance in Rust, as a front end that calls the C library through FFI. The goal of this project was the C library; the UI exists to make it pleasant to use.

## What it does

**Phasor and complex math** (`circuit.c`)
- A `Complex` type that keeps rectangular and polar forms in sync, so values can be entered and read in either form
- Addition, subtraction, multiplication, division, and conjugates
- RMS and peak conversion

**Circuit helpers** (`circuit.c`)
- Impedance of resistors, inductors, and capacitors at a given frequency
- Series and parallel combinations of any number of impedances
- Ohm's law, voltage dividers, and current dividers
- Complex power from V·I\*, |I|²·Z, or |V|²/Z\*, with P, Q, |S|, and leading/lagging power factor

**Solvers** (`analysis.c`)
- **Power triangle:** give any two of P, Q, S, θ, and pf and it fills in the rest
- **Maximum average power transfer:** from a Thevenin voltage or Norton current (peak or RMS) and Zth, finds the matched load (Z_L = Zth\*) and the maximum power delivered

## Terminal UI

The `tui/` front end lists every operation in a menu and walks through the inputs, accepting complex values in either rectangular or polar form. It also includes:
- A power triangle solver with a drawn triangle
- A max power transfer solver
- An impedance builder that adds components one at a time in series or parallel

Controls: `↑`/`↓` or `j`/`k` to navigate, `Enter` to select or solve, `Tab` to move between fields, `Esc` to go back, `q` to quit.

## Building

**Requirements:** a C compiler (`clang` or `gcc`) and, for the UI, the Rust toolchain ([rustup.rs](https://rustup.rs)).

Run the terminal UI (this compiles the C library automatically):

```sh
cd tui
cargo run --release
```

Build the plain C command-line version:

```sh
make
./main
```

The C version runs a single maximum power transfer calculation:

```
Enter Vth (magnitude phase_deg): 18.97 71.56
Enter Impedance (real imag): 0.5 -0.5
Max power to load: 89.9652 W
```

## Conventions

- Phase angles are entered and displayed in **degrees**.
- Power calculations assume **RMS** phasors. Use the RMS/peak conversion if your source is given as a peak value.
- In the power triangle solver, enter a **negative** pf, Q, or θ for a leading (capacitive) load.

## Limitations

This is a personal study tool, so it assumes valid input. Entering things like a zero impedance or a power factor above 1 will give meaningless results rather than an error.

## Project layout

```
circuit.h / circuit.c    Complex type, arithmetic, impedance, dividers, power
analysis.h / analysis.c  Power triangle and max power transfer solvers
main.c                   Minimal C command-line driver
tui/                     Rust terminal UI (ratatui), calls the C library via FFI
```
