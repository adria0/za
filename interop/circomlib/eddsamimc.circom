/*
    Copyright 2018 0KIMS association.

    This file is part of circom (Zero Knowledge Circuit Compiler).

    circom is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    circom is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with circom. If not, see <https://www.gnu.org/licenses/>.
*/

include "compconstant.circom";
include "pointbits.circom";
include "mimc.circom";
include "bitify.circom";
include "escalarmulany.circom";
include "escalarmulfix.circom";

template EdDSAMiMCVerifier() {
    signal input enabled;
    signal input Ax;
    signal input Ay;

    signal input S;
    signal input R8x;
    signal input R8y;

    signal input M;

    var i;

// Ensure S<Subgroup Order

    component snum2bits = Num2Bits(253);
    snum2bits.in <== S;

    component  compConstant = CompConstant(2736030358979909402780800718157159386076813972158567259200215660948447373040);

    for (i=0; i<253; i+=1) {
        snum2bits.out[i] ==> compConstant.in[i];
    }
    compConstant.in[253] <== 0;
    compConstant.out === 0;

// Calculate the h = H(R,A, msg)

    component hash = MultiMiMC7(5, 91);
    hash.in[0] <== R8x;
    hash.in[1] <== R8y;
    hash.in[2] <== Ax;
    hash.in[3] <== Ay;
    hash.in[4] <== M;

    component h2bits = Num2Bits_strict();
    h2bits.in <== hash.out;

// Calculate second part of the right side:  right2 = h*8*A

    // Multiply by 8 by adding it 3 times.  This also ensure that the result is in
    // the subgroup.
    component dbl1 = BabyDbl();
    dbl1.x <== Ax;
    dbl1.y <== Ay;
    component dbl2 = BabyDbl();
    dbl2.x <== dbl1.xout;
    dbl2.y <== dbl1.yout;
    component dbl3 = BabyDbl();
    dbl3.x <== dbl2.xout;
    dbl3.y <== dbl2.yout;

    // We check that A is not zero.
    component isZero = IsZero();
    isZero.in <== dbl3.x;
    isZero.out === 0;

    component mulAny = EscalarMulAny(254);
    for (i=0; i<254; i+=1) {
        mulAny.e[i] <== h2bits.out[i];
    }
    mulAny.p[0] <== dbl3.xout;
    mulAny.p[1] <== dbl3.yout;


// Compute the right side: right =  R8 + right2

    component addRight = BabyAdd();
    addRight.x1 <== R8x;
    addRight.y1 <== R8y;
    addRight.x2 <== mulAny.out[0];
    addRight.y2 <== mulAny.out[1];

// Calculate left side of equation left = S*B8

    var BASE8 = [
        17777552123799933955779906779655732241715742912184938656739573121738514868268,
        2626589144620713026669568689430873010625803728049924121243784502389097019475
    ];
    component mulFix = EscalarMulFix(253, BASE8);
    for (i=0; i<253; i+=1) {
        mulFix.e[i] <== snum2bits.out[i];
    }

// Do the comparation left == right if enabled;

    component eqCheckX = ForceEqualIfEnabled();
    eqCheckX.enabled <== enabled;
    eqCheckX.in[0] <== mulFix.out[0];
    eqCheckX.in[1] <== addRight.xout;

    component eqCheckY = ForceEqualIfEnabled();
    eqCheckY.enabled <== enabled;
    eqCheckY.in[0] <== mulFix.out[1];
    eqCheckY.in[1] <== addRight.yout;
}

#[test]
template test_eddsamimc_verifier() {

    component t = EdDSAMiMCVerifier();
    t.enabled <== 1;
    t.Ax <== 2610057752638682202795145288373380503107623443963127956230801721756904484787;
    t.Ay <== 16617171478497210597712478520507818259149717466230047843969353176573634386897;
    t.R8x <== 1588713735560593324566924232125737788074832889634158814436053427558580690375;
    t.R8y <== 12533855015612340600378032132649667554484548198127330258648136408022725096959;
    t.S <== 719523466698501471009526248132086331983100178909946455948249293695562095591;
    t.M <== 1234;

}
