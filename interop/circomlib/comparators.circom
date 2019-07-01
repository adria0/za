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

include "bitify.circom";
include "binsum.circom";

template IsZero() {
    signal input in;
    signal output out;

    signal inv;
    
    #[w] if (in!=0) {
        inv <-- 1/in;   
    } else {
        inv <-- 0;
    }

    out <== -in*inv +1;
    in*out === 0;
}


template IsEqual() {
    signal input in[2];
    signal output out;

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    isz.out ==> out;
}

template ForceEqualIfEnabled() {
    signal input enabled;
    signal input in[2];

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    (1 - isz.out)*enabled === 0;
}


// N is the number of bits the input  have.
// The MSF is the sign bit.
template LessThan(n) {
    signal input in[2];
    signal output out;

    component num2Bits0;
    component num2Bits1;

    component adder;

    adder = BinSum(n, 2);

    num2Bits0 = Num2Bits(n);
    num2Bits1 = Num2BitsNeg(n);

    in[0] ==> num2Bits0.in;
    in[1] ==> num2Bits1.in;

    var i;
    for (i=0;i<n;i+=1) {
        num2Bits0.out[i] ==> adder.in[0][i];
        num2Bits1.out[i] ==> adder.in[1][i];
    }

    adder.out[n-1] ==> out;
}

#[test]
template test_IsZero_true() {
    component main = IsZero();
    main.in <== 0;
    main.out === 1;
}

#[test]
template test_IsZero_false() {
    component main = IsZero();
    main.in <== 2;
    main.out === 0;
}

#[test]
template test_IsEqual_true() {
    component main = IsEqual();
    main.in[0] <== 2;
    main.in[1] <== 2;
    main.out === 1;
}

#[test]
template test_IsEqual_false() {
    component main = IsEqual();
    main.in[0] <== 2;
    main.in[1] <== 3;
    main.out === 0;
}