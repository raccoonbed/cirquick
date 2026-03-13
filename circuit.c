#include <stdio.h>
#include <math.h>
#include "circuit.h"

#define DEG2RAD (PI / 180.0)
#define RAD2DEG (180.0 / PI)

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

Complex from_rect(double real, double imag) {
    Complex z;
    z.real  = real;
    z.imag  = imag;
    z.mag   = sqrt(real*real + imag*imag);
    z.phase = atan2(imag, real) * RAD2DEG;
    return z;
}

Complex from_polar(double mag, double phase_deg) {
    Complex z;
    z.mag   = mag;
    z.phase = phase_deg;
    z.real  = mag * cos(phase_deg * DEG2RAD);
    z.imag  = mag * sin(phase_deg * DEG2RAD);
    return z;
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

void print_rect(const char *label, Complex z) {
    if (z.imag >= 0)
        printf("%-6s = %8.4f + j%-8.4f\n", label, z.real,  z.imag);
    else
        printf("%-6s = %8.4f - j%-8.4f\n", label, z.real, -z.imag);
}

void print_polar(const char *label, Complex z) {
    printf("%-6s = %8.4f ∠ %.4f°\n", label, z.mag, z.phase);
}

void print_both(const char *label, Complex z) {
    if (z.imag >= 0)
        printf("%-6s = %8.4f + j%-8.4f  =  %.4f ∠ %.4f°\n",
               label, z.real,  z.imag, z.mag, z.phase);
    else
        printf("%-6s = %8.4f - j%-8.4f  =  %.4f ∠ %.4f°\n",
               label, z.real, -z.imag, z.mag, z.phase);
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

Complex cadd(Complex a, Complex b) {
    return from_rect(a.real + b.real, a.imag + b.imag);
}

Complex csub(Complex a, Complex b) {
    return from_rect(a.real - b.real, a.imag - b.imag);
}

Complex cmul(Complex a, Complex b) {
    double real = a.real*b.real - a.imag*b.imag;
    double imag = a.real*b.imag + a.imag*b.real;
    return from_rect(real, imag);
}

Complex cdiv(Complex a, Complex b) {
    double denom = b.real*b.real + b.imag*b.imag;
    double real  = (a.real*b.real + a.imag*b.imag) / denom;
    double imag  = (a.imag*b.real - a.real*b.imag) / denom;
    return from_rect(real, imag);
}

Complex cconj(Complex a) {
    return from_rect(a.real, -a.imag);
}

// ---------------------------------------------------------------------------
// Angular frequency helper
// ---------------------------------------------------------------------------

double omega(double freq_hz) {
    return 2.0 * PI * freq_hz;
}

// ---------------------------------------------------------------------------
// Impedance
// ---------------------------------------------------------------------------

Complex Z_R(double R) {
    return from_rect(R, 0.0);
}

Complex Z_L(double L, double freq_hz) {
    return from_rect(0.0, omega(freq_hz) * L);
}

Complex Z_C(double C, double freq_hz) {
    return from_rect(0.0, -1.0 / (omega(freq_hz) * C));
}

Complex Z_series2(Complex a, Complex b) {
    return cadd(a, b);
}

Complex Z_parallel2(Complex a, Complex b) {
    return cdiv(cmul(a, b), cadd(a, b));
}

Complex Z_series(Complex *arr, int n) {
    Complex total = from_rect(0.0, 0.0);
    for (int i = 0; i < n; i++)
        total = cadd(total, arr[i]);
    return total;
}

Complex Z_parallel(Complex *arr, int n) {
    Complex one   = from_rect(1.0, 0.0);
    Complex total = from_rect(0.0, 0.0);
    for (int i = 0; i < n; i++)
        total = cadd(total, cdiv(one, arr[i]));
    return cdiv(one, total);
}

// ---------------------------------------------------------------------------
// RMS / Peak conversion
// ---------------------------------------------------------------------------

Complex to_rms(Complex peak) {
    return from_polar(peak.mag / SQRT2, peak.phase);
}

Complex to_peak(Complex rms) {
    return from_polar(rms.mag * SQRT2, rms.phase);
}

// ---------------------------------------------------------------------------
// Ohm's Law
// ---------------------------------------------------------------------------

Complex ohm_voltage(Complex I, Complex Z) {
    return cmul(I, Z);
}

Complex ohm_current(Complex V, Complex Z) {
    return cdiv(V, Z);
}

// ---------------------------------------------------------------------------
// Dividers
// ---------------------------------------------------------------------------

Complex voltage_divider(Complex V_s, Complex Z1, Complex Z2) {
    // V_Z2 = V_s * Z2 / (Z1 + Z2)
    return cmul(V_s, cdiv(Z2, cadd(Z1, Z2)));
}

Complex current_divider(Complex I_s, Complex Z1, Complex Z2) {
    // Current through Z2 = I_s * Z1 / (Z1 + Z2)
    return cmul(I_s, cdiv(Z1, cadd(Z1, Z2)));
}

// ---------------------------------------------------------------------------
// Complex Power
// ---------------------------------------------------------------------------

Complex complex_power(Complex V, Complex I) {
    return cmul(V, cconj(I));               // S = V * I*
}

Complex power_from_IZ(Complex I, Complex Z) {
    double i_mag_sq = I.mag * I.mag;        // |I|²
    return from_rect(i_mag_sq * Z.real,     // S = |I|² * Z
                     i_mag_sq * Z.imag);
}

Complex power_from_VZ(Complex V, Complex Z) {
    double v_mag_sq = V.mag * V.mag;        // |V|²
    return cdiv(from_rect(v_mag_sq, 0.0),   // S = |V|² / Z*
                cconj(Z));
}

double real_power(Complex S) {
    return S.real;              // P [W]
}

double reactive_power(Complex S) {
    return S.imag;              // Q [VAR]
}

double apparent_power(Complex S) {
    return S.mag;               // |S| [VA]
}

double power_factor(Complex S) {
    return S.real / S.mag;      // pf = cos(θ)
}

void print_power(const char *label, Complex S) {
    const char *pf_type;
    if      (S.imag > 1e-9)  pf_type = "lagging (inductive)";
    else if (S.imag < -1e-9) pf_type = "leading (capacitive)";
    else                     pf_type = "unity";

    printf("--- %s ---\n", label);
    printf("  S  = %.4f VA  ∠ %.4f°\n", S.mag,  S.phase);
    printf("  P  = %.4f W\n",  S.real);
    printf("  Q  = %.4f VAR\n", S.imag);
    printf("  pf = %.4f  (%s)\n", power_factor(S), pf_type);
}
