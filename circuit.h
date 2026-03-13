#ifndef CIRCUIT_H
#define CIRCUIT_H

#define PI    3.14159265358979323846
#define SQRT2 1.41421356237309504880

// Complex number: rectangular and polar always kept in sync.
// phase is in degrees.
typedef struct {
    double real;    // rectangular: real part
    double imag;    // rectangular: imaginary part
    double mag;     // polar: magnitude
    double phase;   // polar: phase angle (degrees)
} Complex;

// ---------------------------------------------------------------------------
// Constructors — create a Complex from either form, fills both automatically
// ---------------------------------------------------------------------------
Complex from_rect(double real, double imag);
Complex from_polar(double mag, double phase_deg);

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------
void print_rect(const char *label, Complex z);   // "label = a +/- jb"
void print_polar(const char *label, Complex z);  // "label = M ∠ θ°"
void print_both(const char *label, Complex z);   // both on one line

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------
Complex cadd(Complex a, Complex b);   // a + b
Complex csub(Complex a, Complex b);   // a - b
Complex cmul(Complex a, Complex b);   // a * b
Complex cdiv(Complex a, Complex b);   // a / b
Complex cconj(Complex a);             // complex conjugate a*

// ---------------------------------------------------------------------------
// Angular frequency helper
// ---------------------------------------------------------------------------
double omega(double freq_hz);         // ω = 2πf

// ---------------------------------------------------------------------------
// Impedance
// ---------------------------------------------------------------------------
Complex Z_R(double R);                        // Z = R
Complex Z_L(double L, double freq_hz);        // Z = jωL
Complex Z_C(double C, double freq_hz);        // Z = 1/(jωC) = -j/(ωC)

Complex Z_series2(Complex a, Complex b);              // Z = Za + Zb
Complex Z_parallel2(Complex a, Complex b);            // Z = ZaZb / (Za+Zb)
Complex Z_series(Complex *arr, int n);                // Z = sum of n impedances
Complex Z_parallel(Complex *arr, int n);              // 1/Z = sum of 1/Zi

// ---------------------------------------------------------------------------
// RMS / Peak conversion  (phase unchanged, magnitude scaled by 1/√2 or √2)
// NOTE: complex_power() gives correct W/VAR only with RMS phasors.
// ---------------------------------------------------------------------------
Complex to_rms(Complex peak);     // V_rms  = V_peak / √2
Complex to_peak(Complex rms);     // V_peak = V_rms  * √2

// ---------------------------------------------------------------------------
// Ohm's Law
// ---------------------------------------------------------------------------
Complex ohm_voltage(Complex I, Complex Z);    // V = IZ
Complex ohm_current(Complex V, Complex Z);    // I = V/Z

// ---------------------------------------------------------------------------
// Voltage and current dividers
// ---------------------------------------------------------------------------
Complex voltage_divider(Complex V_s, Complex Z1, Complex Z2); // V across Z2
Complex current_divider(Complex I_s, Complex Z1, Complex Z2); // I through Z2

// ---------------------------------------------------------------------------
// Complex Power  (S = P + jQ)
// ---------------------------------------------------------------------------
Complex complex_power(Complex V, Complex I);      // S = V * I*
Complex power_from_IZ(Complex I, Complex Z);      // S = |I|² * Z
Complex power_from_VZ(Complex V, Complex Z);      // S = |V|² / Z*
double  real_power(Complex S);                    // P = Re(S)  [W]
double  reactive_power(Complex S);                // Q = Im(S)  [VAR]
double  apparent_power(Complex S);                // |S|        [VA]
double  power_factor(Complex S);                  // pf = P/|S| = cos(θ)
void    print_power(const char *label, Complex S);

#endif
