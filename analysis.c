#include "analysis.h"
#include "circuit.h"
#include <math.h>

//Power triangle solver helper function
PowerTriangle solve_pt(TriangleInput mode , double x, double y){

    PowerTriangle result;

    switch(mode) {
        case PT_P_Q:
            result.P = x;
            result.Q = y;
            result.S = hypot(x,y);
            result.theta = atan2(y,x);
            result.pf = cos(result.theta);
            break;

        case PT_P_S:
            result.P = x;
            result.Q = sqrt((y*y)-(x*x));
            result.S = y;
            result.theta = atan2(result.Q,x);
            result.pf = x/y;
            break;

        case PT_P_theta:
            result.P = x;
            result.theta = y;
            result.pf = cos(y);
            result.S = x / result.pf;
            result.Q = result.S * sin(y);
            break;

        case PT_P_pf:
            result.pf = y;
            result.theta = acos(y);
            result.P = x;
            result.S = x/y;
            result.Q = result.S * sin(result.theta);
            break;

        case PT_Q_S:
            result.Q = x;
            result.S = y;
            result.P = sqrt((y*y)-(x*x));
            result.theta = atan2(x,result.P);
            result.pf = cos(result.theta);
            break;

        case PT_Q_theta:
            result.Q = x;
            result.theta = y;
            result.pf = cos(result.theta);
            result.P = x/(tan(y));
            result.S = hypot(x,result.P);
            break;

        case PT_Q_pf:
            result.Q = x;
            result.pf = y;
            result.theta = acos(y);
            result.P = x/tan(result.theta);
            result.S = hypot(x, result.P);
            break;

        case PT_S_theta:
            result.S = x;
            result.theta = y;
            result.P = x*cos(y);
            result.Q = x*sin(y);
            result.pf = cos(y);
            break;

        case PT_S_pf:
            result.S = x;
            result.pf = y;
            result.P = x*y;
            result.theta = acos(y);
            result.Q = x*sin(result.theta);
            break;
    }
    return result;
};

double pmax_compute(Complex Vth, Complex Zth, int rms) {
    double mag_Vth = (rms == 1) ? Vth.mag : Vth.mag / SQRT2;
    return (mag_Vth * mag_Vth) / (4.0 * Zth.real);
}

int power_triangle_type(Complex S){

    //Extract reactive part, check if neg or pos
    if (S.imag > 0) {
        return 1;
    }
    else if (S.imag == 0){
        return 0;
    }
    else return -1;
}
