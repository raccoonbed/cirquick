#ifndef ANALYSIS_H
#define ANALYSIS_H
#include "circuit.h"

//----------------Max Avg Power----------------

typedef struct {
    Complex Vth; // thevenin Voltage (V)
    Complex In; // norton Current (A)
    Complex Zth; // thevenin/Norton impedance (Ω)
    Complex Zl; // load impedance (Ω)
    double pmax; // max average power (W)

} MaxAveragePower;

typedef enum {
    MAP_V,
    MAP_I,
    MAP_Vrms,
    MAP_Irms

} MaxAveragePowerInput;

MaxAveragePower solve_ckt(MaxAveragePowerInput mode, Complex x, Complex Z);

//----------------Power Triangle----------------
//Declaring PowerTriangle struct
typedef struct {
    double P; //real power (W)
    double Q; //reactive power (VAr)
    double S; //apparent power (VA)
    double theta; //phase angle (ø)
    double pf; //power factor

} PowerTriangle;

//Enumarating different types of inputs
typedef enum {
    PT_P_Q,
    PT_P_S,
    PT_P_theta,
    PT_P_pf,
    PT_Q_S,
    PT_Q_theta,
    PT_Q_pf,
    PT_S_theta,
    PT_S_pf

} TriangleInput;

PowerTriangle solve_pt(TriangleInput mode , double x, double y);

double pmax_compute(Complex Vth, Complex Zth, int rms);

int power_triangle_type(Complex S);
#endif
