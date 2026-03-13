#include <stdio.h>
#include "analysis.h"
#include "circuit.h"

// ---------------------------------------------------------------------------
// Example problem:
//
//   Vs = 18.97 V ∠ 71.56 (degrees)
//   Series Thevenin: Zth = .5-j.5 series w/ ZL = .5+j.5
//
//   Find: Max power delivered to load
// ---------------------------------------------------------------------------

int main(void) {

    double vmag, vphase, zreal, zimag;

    printf("Enter Vth (magnitude phase_deg): ");
    scanf("%lf %lf", &vmag, &vphase);

    Complex V = from_polar(vmag, vphase);

    printf("\n\nEnter Impedance (real imag): ");
    scanf("%lf %lf", &zreal, &zimag);
    Complex Z = from_rect(zreal, zimag);

    double pmax = pmax_compute(V, Z, 0);
    printf("Max power to load: %.4f W\n", pmax);

    return 0;
}
