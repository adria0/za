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
include "escalarmulfix.circom";

template BabyAdd() {
    signal input x1;
    signal input y1;
    signal input x2;
    signal input y2;
    signal output xout;
    signal output yout;

    signal beta;
    signal gamma;
    signal delta;
    signal tau;

    var a = 168700;
    var d = 168696;

    beta <== x1*y2;
    gamma <== y1*x2;
    delta <== (-a*x1+y1)*(x2 + y2);
    tau <== beta * gamma;

    #[w] xout <-- (beta + gamma) / (1+ d*tau);
    (1+ d*tau) * xout === (beta + gamma);

    #[w] yout <-- (delta + a*beta - gamma) / (1-d*tau);
    (1-d*tau)*yout === (delta + a*beta - gamma);
}

template BabyDbl() {
    signal input x;
    signal input y;
    signal output xout;
    signal output yout;

    component adder = BabyAdd();
    adder.x1 <== x;
    adder.y1 <== y;
    adder.x2 <== x;
    adder.y2 <== y;

    adder.xout ==> xout;
    adder.yout ==> yout;
}

template BabyCheck() {
    signal input x;
    signal input y;

    signal x2;
    signal y2;

    var a = 168700;
    var d = 168696;

    x2 <== x*x;
    y2 <== y*y;

    a*x2 + y2 === 1 + d*x2*y2;
}

// Extracts the public key from private key
template BabyPbk() {
    signal private input  in;
    signal         output Ax;
    signal         output Ay;

    var BASE8 = [
        17777552123799933955779906779655732241715742912184938656739573121738514868268,
        2626589144620713026669568689430873010625803728049924121243784502389097019475
    ];

    component pvkBits = Num2Bits(253);
    pvkBits.in <== in;

    component mulFix = EscalarMulFix(253, BASE8);

    var i;
    for (i=0; i<253; i+=1) {
        mulFix.e[i] <== pvkBits.out[i];
    }
    Ax  <== mulFix.out[0];
    Ay  <== mulFix.out[1];
}

#[test]
template test_BabyAdd_01() {
    component main =  BabyAdd();
    #[w] {
        main.x1 <== 0;
        main.y1 <== 1;
        main.x2 <== 0;
        main.y2 <== 1;

        main.xout === 0;
        main.yout === 1;
    }
}

#[test]
template test_BabyAdd_same() {
    component main =  BabyAdd();

    #[w] {
        main.x1 <== 17777552123799933955779906779655732241715742912184938656739573121738514868268;
        main.y1 <== 2626589144620713026669568689430873010625803728049924121243784502389097019475;
        main.x2 <== 17777552123799933955779906779655732241715742912184938656739573121738514868268;
        main.y2 <== 2626589144620713026669568689430873010625803728049924121243784502389097019475;
        main.xout === 6890855772600357754907169075114257697580319025794532037257385534741338397365;
        main.yout === 4338620300185947561074059802482547481416142213883829469920100239455078257889;
    }
}


#[test]
template test_BabyAdd_different() {
    component main =  BabyAdd();

    #[w] {
        main.x1 <== 17777552123799933955779906779655732241715742912184938656739573121738514868268;
        main.y1 <== 2626589144620713026669568689430873010625803728049924121243784502389097019475;
        main.x2 <== 16540640123574156134436876038791482806971768689494387082833631921987005038935;
        main.y2 <== 20819045374670962167435360035096875258406992893633759881276124905556507972311;
        main.xout === 7916061937171219682591368294088513039687205273691143098332585753343424131937;
        main.yout === 14035240266687799601661095864649209771790948434046947201833777492504781204499;
    }
}
