    final snarkCircuit= """

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

template WindowMulFix() {
    signal input in[3];
    signal input base[2];
    signal output out[2];
    signal output out8[2];   // Returns 8*Base (To be linked)

    component mux = MultiMux3(2);

    mux.s[0] <== in[0];
    mux.s[1] <== in[1];
    mux.s[2] <== in[2];

    component dbl2 = MontgomeryDouble();
    component adr3 = MontgomeryAdd();
    component adr4 = MontgomeryAdd();
    component adr5 = MontgomeryAdd();
    component adr6 = MontgomeryAdd();
    component adr7 = MontgomeryAdd();
    component adr8 = MontgomeryAdd();

// in[0]  -> 1*BASE

    mux.c[0][0] <== base[0];
    mux.c[1][0] <== base[1];

// in[1] -> 2*BASE
    dbl2.in[0] <== base[0];
    dbl2.in[1] <== base[1];
    mux.c[0][1] <== dbl2.out[0];
    mux.c[1][1] <== dbl2.out[1];

// in[2] -> 3*BASE
    adr3.in1[0] <== base[0];
    adr3.in1[1] <== base[1];
    adr3.in2[0] <== dbl2.out[0];
    adr3.in2[1] <== dbl2.out[1];
    mux.c[0][2] <== adr3.out[0];
    mux.c[1][2] <== adr3.out[1];

// in[3] -> 4*BASE
    adr4.in1[0] <== base[0];
    adr4.in1[1] <== base[1];
    adr4.in2[0] <== adr3.out[0];
    adr4.in2[1] <== adr3.out[1];
    mux.c[0][3] <== adr4.out[0];
    mux.c[1][3] <== adr4.out[1];

// in[4] -> 5*BASE
    adr5.in1[0] <== base[0];
    adr5.in1[1] <== base[1];
    adr5.in2[0] <== adr4.out[0];
    adr5.in2[1] <== adr4.out[1];
    mux.c[0][4] <== adr5.out[0];
    mux.c[1][4] <== adr5.out[1];

// in[5] -> 6*BASE
    adr6.in1[0] <== base[0];
    adr6.in1[1] <== base[1];
    adr6.in2[0] <== adr5.out[0];
    adr6.in2[1] <== adr5.out[1];
    mux.c[0][5] <== adr6.out[0];
    mux.c[1][5] <== adr6.out[1];

// in[6] -> 7*BASE
    adr7.in1[0] <== base[0];
    adr7.in1[1] <== base[1];
    adr7.in2[0] <== adr6.out[0];
    adr7.in2[1] <== adr6.out[1];
    mux.c[0][6] <== adr7.out[0];
    mux.c[1][6] <== adr7.out[1];

// in[7] -> 8*BASE
    adr8.in1[0] <== base[0];
    adr8.in1[1] <== base[1];
    adr8.in2[0] <== adr7.out[0];
    adr8.in2[1] <== adr7.out[1];
    mux.c[0][7] <== adr8.out[0];
    mux.c[1][7] <== adr8.out[1];

    out8[0] <== adr8.out[0];
    out8[1] <== adr8.out[1];

    out[0] <== mux.out[0];
    out[1] <== mux.out[1];
}


template SegmentMulFix(nWindows) {
    signal input e[nWindows*3];
    signal input base[2];
    signal output out[2];
    signal output dbl[2];

    var i;
    var j;

    // Convert the base to montgomery

    component e2m = Edwards2Montgomery();
    e2m.in[0] <== base[0];
    e2m.in[1] <== base[1];

    component windows[nWindows];
    component adders[nWindows-1];
    component cadders[nWindows-1];
    for (i=0; i<nWindows; i+=1) {
        windows[i] = WindowMulFix();
        for (j=0; j<3; j+=1) {
            windows[i].in[j] <== e[3*i+j];
        }
        if (i==0) {
            windows[i].base[0] <== e2m.out[0];
            windows[i].base[1] <== e2m.out[1];
        } else {
            windows[i].base[0] <== windows[i-1].out8[0];
            windows[i].base[1] <== windows[i-1].out8[1];

            adders[i-1] = MontgomeryAdd();
            cadders[i-1] = MontgomeryAdd();
            if (i==1) {
                adders[i-1].in1[0] <== windows[0].out[0];
                adders[i-1].in1[1] <== windows[0].out[1];
                cadders[i-1].in1[0] <== e2m.out[0];
                cadders[i-1].in1[1] <== e2m.out[1];
            } else {
                adders[i-1].in1[0] <== adders[i-2].out[0];
                adders[i-1].in1[1] <== adders[i-2].out[1];
                cadders[i-1].in1[0] <== cadders[i-2].out[0];
                cadders[i-1].in1[1] <== cadders[i-2].out[1];
            }
            adders[i-1].in2[0] <== windows[i].out[0];
            adders[i-1].in2[1] <== windows[i].out[1];
            cadders[i-1].in2[0] <== windows[i-1].out8[0];
            cadders[i-1].in2[1] <== windows[i-1].out8[1];
        }
    }

    component m2e = Montgomery2Edwards();
    component cm2e = Montgomery2Edwards();

    if (nWindows > 1) {
        m2e.in[0] <== adders[nWindows-2].out[0];
        m2e.in[1] <== adders[nWindows-2].out[1];
        cm2e.in[0] <== cadders[nWindows-2].out[0];
        cm2e.in[1] <== cadders[nWindows-2].out[1];
    } else {
        m2e.in[0] <== windows[0].out[0];
        m2e.in[1] <== windows[0].out[1];
        cm2e.in[0] <== e2m.out[0];
        cm2e.in[1] <== e2m.out[1];
    }

    component cAdd = BabyAdd();
    cAdd.x1 <== m2e.out[0];
    cAdd.y1 <== m2e.out[1];
    cAdd.x2 <== -cm2e.out[0];
    cAdd.y2 <== cm2e.out[1];


    cAdd.xout ==> out[0];
    cAdd.yout ==> out[1];

    windows[nWindows-1].out8[0] ==> dbl[0];
    windows[nWindows-1].out8[1] ==> dbl[1];
}

template EscalarMulFix(n, BASE) {
    signal input e[n];              // Input in binary format
    signal output out[2];           // Point (Twisted format)

    var nsegments = (n-1)\\249 +1;
    var nlastsegment = n - (nsegments-1)*249;

    component segments[nsegments];

    component m2e[nsegments-1];
    component adders[nsegments-1];

    var s;
    var i;
    var nseg;
    var nWindows;

    for (s=0; s<nsegments; s+=1) {

        if (s < nsegments-1) {
            nseg = 249;
        } else {
            nseg = nlastsegment;
        }

        nWindows = ((nseg - 1)\\3)+1;

        segments[s] = SegmentMulFix(nWindows);

        for (i=0; i<nseg; i+=1) {
            segments[s].e[i] <== e[s*249+i];
        }

        for (i = nseg; i<nWindows*3; i+=1) {
            segments[s].e[i] <== 0;
        }

        if (s==0) {
            segments[s].base[0] <== BASE[0];
            segments[s].base[1] <== BASE[1];
        } else {
            m2e[s-1] = Montgomery2Edwards();
            adders[s-1] = BabyAdd();

            segments[s-1].dbl[0] ==> m2e[s-1].in[0];
            segments[s-1].dbl[1] ==> m2e[s-1].in[1];

            m2e[s-1].out[0] ==> segments[s].base[0];
            m2e[s-1].out[1] ==> segments[s].base[1];

            if (s==1) {
                segments[s-1].out[0] ==> adders[s-1].x1;
                segments[s-1].out[1] ==> adders[s-1].y1;
            } else {
                adders[s-2].xout ==> adders[s-1].x1;
                adders[s-2].yout ==> adders[s-1].y1;
            }
            segments[s].out[0] ==> adders[s-1].x2;
            segments[s].out[1] ==> adders[s-1].y2;
        }
    }

    if (nsegments == 1) {
        segments[0].out[0] ==> out[0];
        segments[0].out[1] ==> out[1];
    } else {
        adders[nsegments-2].xout ==> out[0];
        adders[nsegments-2].yout ==> out[1];
    }
}
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

template EscalarProduct(w) {
    signal input in1[w];
    signal input in2[w];
    signal output out;
    signal aux[w];
    var lc = 0;
    for (var i=0; i<w; i+=1) {
        aux[i] <== in1[i]*in2[i];
        lc = lc + aux[i];
    }
    out <== lc;
}

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


template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1=0;

    for (var i = 0; i<n; i+=1) {
        #[w] out[i] <-- (in >> i) & 1;
        out[i] * (out[i] -1 ) === 0;
        lc1 += out[i] * 2**i;
    }

    lc1 === in;
}

template Num2Bits_strict() {
    signal input in;
    signal output out[254];

    component aliasCheck = AliasCheck();
    component n2b = Num2Bits(254);
    in ==> n2b.in;

    for (var i=0; i<254; i+=1) {
        n2b.out[i] ==> out[i];
        n2b.out[i] ==> aliasCheck.in[i];
    }
}

template Bits2Num(n) {
    signal input in[n];
    signal output out;
    var lc1=0;

    for (var i = 0; i<n; i+=1) {
        lc1 += in[i] * 2**i;
    }

    lc1 ==> out;
}

template Bits2Num_strict() {
    signal input in[n];
    signal output out;

    component aliasCheck = AliasCheck();
    component b2n = Bits2Num(254);

    for (var i=0; i<254; i+=1) {
        in[i] ==> b2n.in[i];
        in[i] ==> aliasCheck.in[i];
    }

    b2n.out ==> out;
}

template Num2BitsNeg(n) {
    signal input in;
    signal output out[n];
    var lc1=0;

    component isZero;

    isZero = IsZero();

    var neg;
    
    if (n == 0) {
         neg = 2**n;
    } else { 
        neg = - in;
    }

    for (var i = 0; i<n; i+=1) {
        out[i] <-- (neg >> i) & 1;
        out[i] * (out[i] -1 ) === 0;
        lc1 += out[i] * 2**i;
    }

    in ==> isZero.in;



    lc1 + isZero.out * 2**n === 2**n - in;
}
template Sign() {
    signal input in[254];
    signal output sign;

    component comp = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);

    var i;

    for (i=0; i<254; i+=1) {
        comp.in[i] <== in[i];
    }

    sign <== comp.out;
}

template EdDSAVerifier(n) {
    signal input msg[n];

    signal input A[256];
    signal input R8[256];
    signal input S[256];

    signal Ax;
    signal Ay;

    signal R8x;
    signal R8y;

    var i;

// Ensure S<Subgroup Order

    component  compConstant = CompConstant(2736030358979909402780800718157159386076813972158567259200215660948447373040);

    for (i=0; i<254; i+=1) {
        S[i] ==> compConstant.in[i];
    }
    compConstant.out === 0;
    S[254] === 0;
    S[255] === 0;

// Convert A to Field elements (And verify A)

    component bits2pointA = Bits2Point_Strict();

    for (i=0; i<256; i+=1) {
        bits2pointA.in[i] <== A[i];
    }
    Ax <== bits2pointA.out[0];
    Ay <== bits2pointA.out[1];

// Convert R8 to Field elements (And verify R8)

    component bits2pointR8 = Bits2Point_Strict();

    for (i=0; i<256; i+=1) {
        bits2pointR8.in[i] <== R8[i];
    }
    R8x <== bits2pointR8.out[0];
    R8y <== bits2pointR8.out[1];

// Calculate the h = H(R,A, msg)

    component hash = Pedersen(512+n);

    for (i=0; i<256; i+=1) {
        hash.in[i] <== R8[i];
        hash.in[256+i] <== A[i];
    }
    for (i=0; i<n; i+=1) {
        hash.in[512+i] <== msg[i];
    }

    component point2bitsH = Point2Bits_Strict();
    point2bitsH.in[0] <== hash.out[0];
    point2bitsH.in[1] <== hash.out[1];

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

    component mulAny = EscalarMulAny(256);
    for (i=0; i<256; i+=1) {
        mulAny.e[i] <== point2bitsH.out[i];
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
    component mulFix = EscalarMulFix(256, BASE8);
    for (i=0; i<256; i+=1) {
        mulFix.e[i] <== S[i];
    }

// Do the comparation left == right

    mulFix.out[0] === addRight.xout;
    mulFix.out[1] === addRight.yout;
}

template Window4() {
    signal input in[4];
    signal input base[2];
    signal output out[2];
    signal output out8[2];   // Returns 8*Base (To be linked)

    component mux = MultiMux3(2);

    mux.s[0] <== in[0];
    mux.s[1] <== in[1];
    mux.s[2] <== in[2];

    component dbl2 = MontgomeryDouble();
    component adr3 = MontgomeryAdd();
    component adr4 = MontgomeryAdd();
    component adr5 = MontgomeryAdd();
    component adr6 = MontgomeryAdd();
    component adr7 = MontgomeryAdd();
    component adr8 = MontgomeryAdd();

// in[0]  -> 1*BASE

    mux.c[0][0] <== base[0];
    mux.c[1][0] <== base[1];

// in[1] -> 2*BASE
    dbl2.in[0] <== base[0];
    dbl2.in[1] <== base[1];
    mux.c[0][1] <== dbl2.out[0];
    mux.c[1][1] <== dbl2.out[1];

// in[2] -> 3*BASE
    adr3.in1[0] <== base[0];
    adr3.in1[1] <== base[1];
    adr3.in2[0] <== dbl2.out[0];
    adr3.in2[1] <== dbl2.out[1];
    mux.c[0][2] <== adr3.out[0];
    mux.c[1][2] <== adr3.out[1];

// in[3] -> 4*BASE
    adr4.in1[0] <== base[0];
    adr4.in1[1] <== base[1];
    adr4.in2[0] <== adr3.out[0];
    adr4.in2[1] <== adr3.out[1];
    mux.c[0][3] <== adr4.out[0];
    mux.c[1][3] <== adr4.out[1];

// in[4] -> 5*BASE
    adr5.in1[0] <== base[0];
    adr5.in1[1] <== base[1];
    adr5.in2[0] <== adr4.out[0];
    adr5.in2[1] <== adr4.out[1];
    mux.c[0][4] <== adr5.out[0];
    mux.c[1][4] <== adr5.out[1];

// in[5] -> 6*BASE
    adr6.in1[0] <== base[0];
    adr6.in1[1] <== base[1];
    adr6.in2[0] <== adr5.out[0];
    adr6.in2[1] <== adr5.out[1];
    mux.c[0][5] <== adr6.out[0];
    mux.c[1][5] <== adr6.out[1];

// in[6] -> 7*BASE
    adr7.in1[0] <== base[0];
    adr7.in1[1] <== base[1];
    adr7.in2[0] <== adr6.out[0];
    adr7.in2[1] <== adr6.out[1];
    mux.c[0][6] <== adr7.out[0];
    mux.c[1][6] <== adr7.out[1];

// in[7] -> 8*BASE
    adr8.in1[0] <== base[0];
    adr8.in1[1] <== base[1];
    adr8.in2[0] <== adr7.out[0];
    adr8.in2[1] <== adr7.out[1];
    mux.c[0][7] <== adr8.out[0];
    mux.c[1][7] <== adr8.out[1];

    out8[0] <== adr8.out[0];
    out8[1] <== adr8.out[1];
    out[0] <== mux.out[0];
    out[1] <== - mux.out[1]*2*in[3] + mux.out[1];  // Negate y if in[3] is one
}


template Segment(nWindows) {
    signal input in[nWindows*4];
    signal input base[2];
    signal output out[2];

    var i;
    var j;

    // Convert the base to montgomery

    component e2m = Edwards2Montgomery();
    e2m.in[0] <== base[0];
    e2m.in[1] <== base[1];

    component windows[nWindows];
    component doublers1[nWindows-1];
    component doublers2[nWindows-1];
    component adders[nWindows-1];
    for (i=0; i<nWindows; i+=1) {
        windows[i] = Window4();
        for (j=0; j<4; j+=1) {
            windows[i].in[j] <== in[4*i+j];
        }
        if (i==0) {
            windows[i].base[0] <== e2m.out[0];
            windows[i].base[1] <== e2m.out[1];
        } else {
            doublers1[i-1] = MontgomeryDouble();
            doublers2[i-1] = MontgomeryDouble();
            doublers1[i-1].in[0] <== windows[i-1].out8[0];
            doublers1[i-1].in[1] <== windows[i-1].out8[1];
            doublers2[i-1].in[0] <== doublers1[i-1].out[0];
            doublers2[i-1].in[1] <== doublers1[i-1].out[1];

            windows[i].base[0] <== doublers2[i-1].out[0];
            windows[i].base[1] <== doublers2[i-1].out[1];

            adders[i-1] = MontgomeryAdd();
            if (i==1) {
                adders[i-1].in1[0] <== windows[0].out[0];
                adders[i-1].in1[1] <== windows[0].out[1];
            } else {
                adders[i-1].in1[0] <== adders[i-2].out[0];
                adders[i-1].in1[1] <== adders[i-2].out[1];
            }
            adders[i-1].in2[0] <== windows[i].out[0];
            adders[i-1].in2[1] <== windows[i].out[1];
        }
    }

    component m2e = Montgomery2Edwards();

    if (nWindows > 1) {
        m2e.in[0] <== adders[nWindows-2].out[0];
        m2e.in[1] <== adders[nWindows-2].out[1];
    } else {
        m2e.in[0] <== windows[0].out[0];
        m2e.in[1] <== windows[0].out[1];
    }

    out[0] <== m2e.out[0];
    out[1] <== m2e.out[1];
}
template Pedersen(n) {
    signal input in[n];
    signal output out[2];

    var BASE = [
        [10457101036533406547632367118273992217979173478358440826365724437999023779287,19824078218392094440610104313265183977899662750282163392862422243483260492317],
        [2671756056509184035029146175565761955751135805354291559563293617232983272177,2663205510731142763556352975002641716101654201788071096152948830924149045094],
        [5802099305472655231388284418920769829666717045250560929368476121199858275951,5980429700218124965372158798884772646841287887664001482443826541541529227896],
        [7107336197374528537877327281242680114152313102022415488494307685842428166594,2857869773864086953506483169737724679646433914307247183624878062391496185654],
        [20265828622013100949498132415626198973119240347465898028410217039057588424236,1160461593266035632937973507065134938065359936056410650153315956301179689506],
        [1487999857809287756929114517587739322941449154962237464737694709326309567994,14017256862867289575056460215526364897734808720610101650676790868051368668003],
        [14618644331049802168996997831720384953259095788558646464435263343433563860015,13115243279999696210147231297848654998887864576952244320558158620692603342236],
        [6814338563135591367010655964669793483652536871717891893032616415581401894627,13660303521961041205824633772157003587453809761793065294055279768121314853695],
        [3571615583211663069428808372184817973703476260057504149923239576077102575715,11981351099832644138306422070127357074117642951423551606012551622164230222506],
        [18597552580465440374022635246985743886550544261632147935254624835147509493269,6753322320275422086923032033899357299485124665258735666995435957890214041481]
    ];

    var nSegments = ((n-1)\\200)+1;

    component segments[nSegments];

    var i;
    var j;
    var nBits;
    var nWindows;

    for (i=0; i<nSegments; i+=1) {
        if (i == (nSegments-1)) {
            nBits = n - (nSegments-1)*200;
        } else {
            nBits = 200;
        }
        nWindows = ((nBits - 1)\\4)+1;
        segments[i] = Segment(nWindows);
        segments[i].base[0] <== BASE[i][0];
        segments[i].base[1] <== BASE[i][1];
        for (j = 0; j<nBits; j+=1) {
            segments[i].in[j] <== in[i*200+j];
        }
        // Fill padding bits
        for (j = nBits; j < nWindows*4; j+=1) {
            segments[i].in[j] <== 0;
        }
    }

    component adders[nSegments-1];

    for (i=0; i<nSegments-1; i+=1) {
        adders[i] = BabyAdd();
        if (i==0) {
            adders[i].x1 <== segments[0].out[0];
            adders[i].y1 <== segments[0].out[1];
            adders[i].x2 <== segments[1].out[0];
            adders[i].y2 <== segments[1].out[1];
        } else {
            adders[i].x1 <== adders[i-1].xout;
            adders[i].y1 <== adders[i-1].yout;
            adders[i].x2 <== segments[i+1].out[0];
            adders[i].y2 <== segments[i+1].out[1];
        }
    }

    if (nSegments>1) {
        out[0] <== adders[nSegments-2].xout;
        out[1] <== adders[nSegments-2].yout;
    } else {
        out[0] <== segments[0].out[0];
        out[1] <== segments[0].out[1];
    }

}

template pedersen256_helper() {
    signal input in;
    signal output out[2];

    component pedersen = Pedersen(256);

    component n2b;
    n2b = Num2Bits(253);

    var i;

    in ==> n2b.in;

    for  (i=0; i<253; i+=1) {
        pedersen.in[i] <== n2b.out[i];
    }

    for (i=253; i<256; i+=1) {
        pedersen.in[i] <== 0;
    }

    pedersen.out[0] ==> out[0];
    pedersen.out[1] ==> out[1];
}

template Multiplexor2() {
    signal input sel;
    signal input in[2][2];
    signal output out[2];

    out[0] <== (in[1][0] - in[0][0])*sel + in[0][0];
    out[1] <== (in[1][1] - in[0][1])*sel + in[0][1];
}

template BitElementMulAny() {
    signal input sel;
    signal input dblIn[2];
    signal input addIn[2];
    signal output dblOut[2];
    signal output addOut[2];

    component doubler = MontgomeryDouble();
    component adder = MontgomeryAdd();
    component selector = Multiplexor2();


    sel ==> selector.sel;

    dblIn[0] ==> doubler.in[0];
    dblIn[1] ==> doubler.in[1];
    doubler.out[0] ==> adder.in1[0];
    doubler.out[1] ==> adder.in1[1];
    addIn[0] ==> adder.in2[0];
    addIn[1] ==> adder.in2[1];
    addIn[0] ==> selector.in[0][0];
    addIn[1] ==> selector.in[0][1];
    adder.out[0] ==> selector.in[1][0];
    adder.out[1] ==> selector.in[1][1];

    doubler.out[0] ==> dblOut[0];
    doubler.out[1] ==> dblOut[1];
    selector.out[0] ==> addOut[0];
    selector.out[1] ==> addOut[1];
}

// p is montgomery point
// n must be <= 248
// returns out in twisted edwards
// Double is in montgomery to be linked;
template SegmentMulAny(n) {
    signal input e[n];
    signal input p[2];
    signal output out[2];
    signal output dbl[2];

    component bits[n-1];

    component e2m = Edwards2Montgomery();

    p[0] ==> e2m.in[0];
    p[1] ==> e2m.in[1];

    var i;

    bits[0] = BitElementMulAny();
    e2m.out[0] ==> bits[0].dblIn[0];
    e2m.out[1] ==> bits[0].dblIn[1];
    e2m.out[0] ==> bits[0].addIn[0];
    e2m.out[1] ==> bits[0].addIn[1];
    e[1] ==> bits[0].sel;

    for (i=1; i<n-1; i+=1) {
        bits[i] = BitElementMulAny();

        bits[i-1].dblOut[0] ==> bits[i].dblIn[0];
        bits[i-1].dblOut[1] ==> bits[i].dblIn[1];
        bits[i-1].addOut[0] ==> bits[i].addIn[0];
        bits[i-1].addOut[1] ==> bits[i].addIn[1];
        e[i+1] ==> bits[i].sel;
    }

    bits[n-2].dblOut[0] ==> dbl[0];
    bits[n-2].dblOut[1] ==> dbl[1];

    component m2e = Montgomery2Edwards();

    bits[n-2].addOut[0] ==> m2e.in[0];
    bits[n-2].addOut[1] ==> m2e.in[1];

    component eadder = BabyAdd();

    m2e.out[0] ==> eadder.x1;
    m2e.out[1] ==> eadder.y1;
    -p[0] ==> eadder.x2;
    p[1] ==> eadder.y2;

    component lastSel = Multiplexor2();

    e[0] ==> lastSel.sel;
    eadder.xout ==> lastSel.in[0][0];
    eadder.yout ==> lastSel.in[0][1];
    m2e.out[0] ==> lastSel.in[1][0];
    m2e.out[1] ==> lastSel.in[1][1];

    lastSel.out[0] ==> out[0];
    lastSel.out[1] ==> out[1];
}

// This function assumes that p is in the subgroup and it is different to 0

template EscalarMulAny(n) {
    signal input e[n];              // Input in binary format
    signal input p[2];              // Point (Twisted format)
    signal output out[2];           // Point (Twisted format)

    var nsegments = (n-1)\\148 +1;
    var nlastsegment = n - (nsegments-1)*148;

    component segments[nsegments];
    component doublers[nsegments-1];
    component m2e[nsegments-1];
    component adders[nsegments-1];

    var s;
    var i;
    var nseg;

    for (s=0; s<nsegments; s+=1) {

        if (s < nsegments-1) {
            nseg = 148 ;
        } else {
            nseg = nlastsegment;
        }
        segments[s] = SegmentMulAny(nseg);

        for (i=0; i<nseg; i+=1) {
            e[s*148+i] ==> segments[s].e[i];
        }

        if (s==0) {
            p[0] ==> segments[s].p[0];
            p[1] ==> segments[s].p[1];
        } else {
            doublers[s-1] = MontgomeryDouble();
            m2e[s-1] = Montgomery2Edwards();
            adders[s-1] = BabyAdd();

            segments[s-1].dbl[0] ==> doublers[s-1].in[0];
            segments[s-1].dbl[1] ==> doublers[s-1].in[1];

            doublers[s-1].out[0] ==> m2e[s-1].in[0];
            doublers[s-1].out[1] ==> m2e[s-1].in[1];

            m2e[s-1].out[0] ==> segments[s].p[0];
            m2e[s-1].out[1] ==> segments[s].p[1];

            if (s==1) {
                segments[s-1].out[0] ==> adders[s-1].x1;
                segments[s-1].out[1] ==> adders[s-1].y1;
            } else {
                adders[s-2].xout ==> adders[s-1].x1;
                adders[s-2].yout ==> adders[s-1].y1;
            }
            segments[s].out[0] ==> adders[s-1].x2;
            segments[s].out[1] ==> adders[s-1].y2;
        }
    }

    if (nsegments == 1) {
        segments[0].out[0] ==> out[0];
        segments[0].out[1] ==> out[1];
    } else {
        adders[nsegments-2].xout ==> out[0];
        adders[nsegments-2].yout ==> out[1];
    }
}

template MultiMux4(n) {
    signal input c[n][16];  // Constants
    signal input s[4];   // Selector
    signal output out[n];

    signal a3210[n];
    signal a321[n];
    signal a320[n];
    signal a310[n];
    signal a32[n];
    signal a31[n];
    signal a30[n];
    signal a3[n];

    signal a210[n];
    signal a21[n];
    signal a20[n];
    signal a10[n];
    signal a2[n];
    signal a1[n];
    signal a0[n];
    signal a[n];

    // 4 constrains for the intermediary variables
    signal  s10;
    s10 <== s[1] * s[0];
    signal  s20;
    s20 <== s[2] * s[0];
    signal  s21;
    s21 <== s[2] * s[1];
    signal s210;
    s210 <==  s21 * s[0];


    for (var i=0; i<n; i+=1) {

        a3210[i] <==  ( c[i][15]-c[i][14]-c[i][13]+c[i][12] - c[i][11]+c[i][10]+c[i][ 9]-c[i][ 8]
                       -c[i][ 7]+c[i][ 6]+c[i][ 5]-c[i][ 4] + c[i][ 3]-c[i][ 2]-c[i][ 1]+c[i][ 0] ) * s210;
         a321[i] <==  ( c[i][14]-c[i][12]-c[i][10]+c[i][ 8] - c[i][ 6]+c[i][ 4]+c[i][ 2]-c[i][ 0] ) * s21;
         a320[i] <==  ( c[i][13]-c[i][12]-c[i][ 9]+c[i][ 8] - c[i][ 5]+c[i][ 4]+c[i][ 1]-c[i][ 0] ) * s20;
         a310[i] <==  ( c[i][11]-c[i][10]-c[i][ 9]+c[i][ 8] - c[i][ 3]+c[i][ 2]+c[i][ 1]-c[i][ 0] ) * s10;
          a32[i] <==  ( c[i][12]-c[i][ 8]-c[i][ 4]+c[i][ 0] ) * s[2];
          a31[i] <==  ( c[i][10]-c[i][ 8]-c[i][ 2]+c[i][ 0] ) * s[1];
          a30[i] <==  ( c[i][ 9]-c[i][ 8]-c[i][ 1]+c[i][ 0] ) * s[0];
           a3[i] <==  ( c[i][ 8]-c[i][ 0] );

         a210[i] <==  ( c[i][ 7]-c[i][ 6]-c[i][ 5]+c[i][ 4] - c[i][ 3]+c[i][ 2]+c[i][ 1]-c[i][ 0] ) * s210;
          a21[i] <==  ( c[i][ 6]-c[i][ 4]-c[i][ 2]+c[i][ 0] ) * s21;
          a20[i] <==  ( c[i][ 5]-c[i][ 4]-c[i][ 1]+c[i][ 0] ) * s20;
          a10[i] <==  ( c[i][ 3]-c[i][ 2]-c[i][ 1]+c[i][ 0] ) * s10;
           a2[i] <==  ( c[i][ 4]-c[i][ 0] ) * s[2];
           a1[i] <==  ( c[i][ 2]-c[i][ 0] ) * s[1];
           a0[i] <==  ( c[i][ 1]-c[i][ 0] ) * s[0];
            a[i] <==  ( c[i][ 0] );

          out[i] <== ( a3210[i] + a321[i] + a320[i] + a310[i] + a32[i] + a31[i] + a30[i] + a3[i] ) * s[3] +
                     (  a210[i] +  a21[i] +  a20[i] +  a10[i] +  a2[i] +  a1[i] +  a0[i] +  a[i] );

    }
}

template Mux4() {
    var i;
    signal input c[16];  // Constants
    signal input s[4];   // Selector
    signal output out;

    component mux = MultiMux4(1);

    for (i=0; i<16; i+=1) {
        mux.c[0][i] <== c[i];
    }

    for (i=0; i<4; i+=1) {
      s[i] ==> mux.s[i];
    }

    mux.out[0] ==> out;
}
template BinSub(n) {
    signal input in[2][n];
    signal output out[n];

    signal aux;

    var lin = 2**n;
    var lout = 0;

    for (var i=0; i<n; i+=1) {
        lin = lin + in[0][i]*(2**i);
        lin = lin - in[1][i]*(2**i);
    }

    for (var i=0; i<n; i+=1) {
        out[i] <-- (lin >> i) & 1;

        // Ensure out is binary
        out[i] * (out[i] - 1) === 0;

        lout = lout + out[i]*(2**i);
    }

    aux <-- (lin >> n) & 1;
    aux*(aux-1) === 0;
    lout = lout + aux*(2**n);

    // Ensure the sum;
    lin === lout;
}

function sqrt(n) {
    if (n == 0) {
        return 0;
    }
    // Test that have solution
    var res = n ** ((-1) >> 1);
    if (res!=1) {
        return 0;
    }
    var m = 28;
    var c = 19103219067921713944291392827692070036145651957329286315305642004821462161904;
    var t = n ** 81540058820840996586704275553141814055101440848469862132140264610111;
    var r = n ** ((81540058820840996586704275553141814055101440848469862132140264610111+1)>>1);
    var sq;
    var i;
    var b;
    var j;

    while ((r != 0)&&(t != 1)) {
        sq = t*t;
        i = 1;
        while (sq!=1) {
            i+=1;
            sq = sq*sq;
        }
        // b = c ^ m-i-1
        b = c;
        for (j=0; j< m-i-1; j+=1)  {
            b = b*b;
        } 

        m = i;
        c = b*b;
        t = t*c;
        r = r*b;
    }
    if (r > ((-1) >> 1)) {
        r = -r;
    }

    return r;

}


template Bits2Point() {
    signal input in[256];
    signal output out[2];
}
template Bits2Point_Strict() {
    signal input in[256];
    signal output out[2];

    var i;

    // Check aliasing
    component aliasCheckY = AliasCheck();
    for (i=0; i<254; i+=1) {
        aliasCheckY.in[i] <== in[i];
    }
    in[254] === 0;

    component b2nY = Bits2Num(254);
    for (i=0; i<254; i+=1) {
        b2nY.in[i] <== in[i];
    }

    out[1] <== b2nY.out;

    #[w] {
        var a = 168700;
        var d = 168696;

        var y2 = out[1] * out[1];
        var x = sqrt(   (1-y2)/(a - d*y2)  );
    
        if (in[255] == 1) {
            x = -x;
        }
        out[0] <-- x;
    }

    component babyCheck = BabyCheck();
    babyCheck.x <== out[0];
    babyCheck.y <== out[1];

    component n2bX = Num2Bits(254);
    n2bX.in <== out[0];
    component aliasCheckX = AliasCheck();
    for (i=0; i<254; i+=1) {
        aliasCheckX.in[i] <== n2bX.out[i];
    }

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    for (i=0; i<254; i+=1) {
        signCalc.in[i] <== n2bX.out[i];
    }

    signCalc.out === in[255];
}




template Point2Bits() {
    signal input in[2];
    signal output out[256];


}

template Point2Bits_Strict() {
    signal input in[2];
    signal output out[256];

    var i;

    component n2bX = Num2Bits(254);
    n2bX.in <== in[0];
    component n2bY = Num2Bits(254);
    n2bY.in <== in[1];

    component aliasCheckX = AliasCheck();
    component aliasCheckY = AliasCheck();
    for (i=0; i<254; i+=1) {
        aliasCheckX.in[i] <== n2bX.out[i];
        aliasCheckY.in[i] <== n2bY.out[i];
    }

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    for (i=0; i<254; i+=1) {
        signCalc.in[i] <== n2bX.out[i];
    }

    for (i=0; i<254; i+=1) {
        out[i] <== n2bY.out[i];
    }
    out[254] <== 0;
    out[255] <== signCalc.out;
}

template EscalarMulWindow(base, k) {

    signal input in[2];
    signal input sel[4];
    signal output out[2];

    component table;
    component mux;
    component adder;

    var i;

    table = EscalarMulW4Table(base, k);
    mux = MultiMux4(2);
    adder = BabyAdd();

    for (i=0; i<4; i+=1) {
        sel[i] ==> mux.s[i];
    }

    for (i=0; i<16; i+=1) {
        table.out[i][0] ==> mux.c[0][i];
        table.out[i][1] ==> mux.c[1][i];
    }

    in[0] ==> adder.x1;
    in[1] ==> adder.y1;

    mux.out[0] ==> adder.x2;
    mux.out[1] ==> adder.y2;

    adder.xout ==> out[0];
    adder.yout ==> out[1];
}
template EscalarMul(n, base) {
    signal input in[n];
    signal input inp[2];   // Point input to be added
    signal output out[2];

    var nBlocks = ((n-1)>>2)+1;
    var i;
    var j;

    component windows[nBlocks];

    // Construct the windows
    for (i=0; i<nBlocks; i+=1) {
      windows[i] = EscalarMulWindow(base, i);
    }

    // Connect the selectors
    for (i=0; i<nBlocks; i+=1) {
        for (j=0; j<4; j+=1) {
            if (i*4+j >= n) {
                windows[i].sel[j] <== 0;
            } else {
                windows[i].sel[j] <== in[i*4+j];
            }
        }
    }

    // Start with generator
    windows[0].in[0] <== inp[0];
    windows[0].in[1] <== inp[1];

    for(i=0; i<nBlocks-1; i+=1) {
        windows[i].out[0] ==> windows[i+1].in[0];
        windows[i].out[1] ==> windows[i+1].in[1];
    }

    windows[nBlocks-1].out[0] ==> out[0];
    windows[nBlocks-1].out[1] ==> out[1];
}

template AliasCheck() {

    signal input in[254];

    component  compConstant = CompConstant(-1);

    for (var i=0; i<254; i+=1) {
        in[i] ==> compConstant.in[i];
    }
    compConstant.out === 0;
}

template CompConstant(ct) {
    signal input in[254];
    signal output out;

    signal parts[127];
    signal sout;

    var clsb;
    var cmsb;
    var slsb;
    var smsb;

    var sum=0;

    var b = (1 << 128) -1;
    var a = 1;
    var e = 1;
    var i;

    for (i=0;i<127; i+=1) {
        clsb = (ct >> (i*2)) & 1;
        cmsb = (ct >> (i*2+1)) & 1;
        slsb = in[i*2];
        smsb = in[i*2+1];


        if ((cmsb==0)&&(clsb==0)) {
            parts[i] <== -b*smsb*slsb + b*smsb + b*slsb;
        } else if ((cmsb==0)&&(clsb==1)) {
            parts[i] <== a*smsb*slsb - a*slsb + b*smsb - a*smsb + a;
        } else if ((cmsb==1)&&(clsb==0)) {
            parts[i] <== b*smsb*slsb - a*smsb + a;
        } else {
            parts[i] <== -a*smsb*slsb + a;
        }

        sum = sum + parts[i];

        b = b -e;
        a = a +e;
        e = e*2;
    }

    sout <== sum;

    component num2bits = Num2Bits(135);

    num2bits.in <== sout;

    out <== num2bits.out[127];
}
template XOR() {
    signal input a;
    signal input b;
    signal output out;

    out <== a + b - 2*a*b;
}

template AND() {
    signal input a;
    signal input b;
    signal output out;

    out <== a*b;
}

template OR() {
    signal input a;
    signal input b;
    signal output out;

    out <== a + b - a*b;
}

template NOT() {
    signal input in;
    signal output out;

    out <== 1 + in - 2*in;
}

template NAND() {
    signal input a;
    signal input b;
    signal output out;

    out <== 1 - a*b;
}

template NOR() {
    signal input a;
    signal input b;
    signal output out;

    out <== a*b + 1 - a - b;
}

template MultiAND(n) {
    signal input in[n];
    signal output out;
    if (n==1) {
        out <== in[0];
    } else if (n==2) {
        component and1 = AND();
        and1.a <== in[0];
        and1.b <== in[1];
        out <== and1.out;
    } else {
        component and2 = AND();
        component ands[2];
        var n1 = n\\2;
        var n2 = n-n\\2;
        ands[0] = MultiAND(n1);
        ands[1] = MultiAND(n2);
        for (var i=0; i<n1; i+=1) {
            ands[0].in[i] <== in[i];
        }
        for (var i=0; i<n2; i+=1) {
            ands[1].in[i] <== in[n1+i];
        }
        and2.a <== ands[0].out;
        and2.b <== ands[1].out;
        out <== and2.out;
    }
}


template MultiMux3(n) {
    signal input c[n][8];  // Constants
    signal input s[3];   // Selector
    signal output out[n];

    signal a210[n];
    signal a21[n];
    signal a20[n];
    signal a2[n];

    signal a10[n];
    signal a1[n];
    signal a0[n];
    signal a[n];

    // 4 constrains for the intermediary variables
    signal  s10;
    s10 <== s[1] * s[0];
    for (var i=0; i<n; i+=1) {

         a210[i] <==  ( c[i][ 7]-c[i][ 6]-c[i][ 5]+c[i][ 4] - c[i][ 3]+c[i][ 2]+c[i][ 1]-c[i][ 0] ) * s10;
          a21[i] <==  ( c[i][ 6]-c[i][ 4]-c[i][ 2]+c[i][ 0] ) * s[1];
          a20[i] <==  ( c[i][ 5]-c[i][ 4]-c[i][ 1]+c[i][ 0] ) * s[0];
           a2[i] <==  ( c[i][ 4]-c[i][ 0] );

          a10[i] <==  ( c[i][ 3]-c[i][ 2]-c[i][ 1]+c[i][ 0] ) * s10;
           a1[i] <==  ( c[i][ 2]-c[i][ 0] ) * s[1];
           a0[i] <==  ( c[i][ 1]-c[i][ 0] ) * s[0];
            a[i] <==  ( c[i][ 0] );

          out[i] <== ( a210[i] + a21[i] + a20[i] + a2[i] ) * s[2] +
                     (  a10[i] +  a1[i] +  a0[i] +  a[i] );

    }
}

template Mux3() {
    var i;
    signal input c[8];  // Constants
    signal input s[3];   // Selector
    signal output out;

    component mux = MultiMux3(1);

    for (i=0; i<8; i+=1) {
        mux.c[0][i] <== c[i];
    }

    for (i=0; i<3; i+=1) {
      s[i] ==> mux.s[i];
    }

    mux.out[0] ==> out;
}

/*
    Source: https://en.wikipedia.org/wiki/Montgomery_curve

                1 + y       1 + y
    [u, v] = [ -------  , ---------- ]
                1 - y      (1 - y)x

 */

template Edwards2Montgomery() {
    signal input in[2];
    signal output out[2];

    #[w] out[0] <-- (1 + in[1]) / (1 - in[1]);
    #[w] out[1] <-- out[0] / in[0];

    out[0] * (1-in[1]) === (1 + in[1]);
    out[1] * in[0] === out[0];
}

/*

                u    u - 1
    [x, y] = [ ---, ------- ]
                v    u + 1

 */
template Montgomery2Edwards() {
    signal input in[2];
    signal output out[2];

    #[w] out[0] <-- in[0] / in[1];
    #[w] out[1] <-- (in[0] - 1) / (in[0] + 1);

    out[0] * in[1] === in[0];
    out[1] * (in[0] + 1) === in[0] - 1;
}


/*
             x2 - x1
    lamda = ---------
             y2 - y1

                                                    x3 + A + x1 + x2
    x3 = B * lamda^2 - A - x1 -x2    =>  lamda^2 = ------------------
                                                         B

    y3 = (2*x1 + x2 + A)*lamda - B*lamda^3 - y1  =>


    =>  y3 = lamda * ( 2*x1 + x2 + A  - x3 - A - x1 - x2)  - y1 =>

    =>  y3 = lamda * ( x1 - x3 ) - y1

----------

             y2 - y1
    lamda = ---------
             x2 - x1

    x3 = B * lamda^2 - A - x1 -x2

    y3 = lamda * ( x1 - x3 ) - y1

 */

template MontgomeryAdd() {
    signal input in1[2];
    signal input in2[2];
    signal output out[2];

    var a = 168700;
    var d = 168696;

    var A = (2 * (a + d)) / (a - d);
    var B = 4 / (a - d);

    signal lamda;

    #[w] lamda <-- (in2[1] - in1[1]) / (in2[0] - in1[0]);
    lamda * (in2[0] - in1[0]) === (in2[1] - in1[1]);

    out[0] <== B*lamda*lamda - A - in1[0] -in2[0];
    out[1] <== lamda * (in1[0] - out[0]) - in1[1];
}

/*

    x1_2 = x1*x1

             3*x1_2 + 2*A*x1 + 1
    lamda = ---------------------
                   2*B*y1

    x3 = B * lamda^2 - A - x1 -x1

    y3 = lamda * ( x1 - x3 ) - y1

 */
template MontgomeryDouble() {
    signal input in[2];
    signal output out[2];

    var a = 168700;
    var d = 168696;

    var A = (2 * (a + d)) / (a - d);
    var B = 4 / (a - d);

    signal lamda;
    signal x1_2;

    x1_2 <== in[0] * in[0];

    #[w] lamda <-- (3*x1_2 + 2*A*in[0] + 1 ) / (2*B*in[1]);
    lamda * (2*B*in[1]) === (3*x1_2 + 2*A*in[0] + 1 );

    out[0] <== B*lamda*lamda - A - 2*in[0];
    out[1] <== lamda * (in[0] - out[0]) - in[1];
}


function nbits(a) {
    var n = 1;
    var r = 0;
    while (n-1<a) {
        r+=1;
        n *= 2;
    }
    return r;
}


template BinSum(n, ops) {
    signal input in[ops][n];
    signal output out[nbits((2**n -1)*ops)];

    var nout = nbits((2**n -1)*ops);

    var lin = 0;
    var lout = 0;

    var k;
    var j;

    for (k=0; k<n; k+=1) {
        for (j=0; j<ops; j+=1) {
            lin += in[j][k] * 2**k;
        }
    }

    for (k=0; k<nout; k+=1) {
        #[w] out[k] <-- (lin >> k) & 1;

        // Ensure out is binary
        out[k] * (out[k] - 1) === 0;

        lout += out[k] * 2**k;
    }

    // Ensure the sum;

    lin === lout;
}

template SMTProcessorSM() {
  signal input xor;
  signal input is0;
  signal input levIns;
  signal input fnc[2];

  signal input prev_top;
  signal input prev_old0;
  signal input prev_bot;
  signal input prev_new1;
  signal input prev_na;
  signal input prev_upd;

  signal output st_top;
  signal output st_old0;
  signal output st_bot;
  signal output st_new1;
  signal output st_na;
  signal output st_upd;

  signal aux1;
  signal aux2;

  aux1 <==                  prev_top * levIns;
  aux2 <== aux1*fnc[0];  // prev_top * levIns * fnc[0]

  // st_top = prev_top*(1-levIns)
  //    = + prev_top
  //      - prev_top * levIns            = aux1

  st_top <== prev_top - aux1;

  // st_old0 = prev_top * levIns * is0 * fnc[0]
  //      = + prev_top * levIns * is0 * fnc[0]            = aux2 * is0

  st_old0 <== aux2 * is0;  // prev_top * levIns * is0 * fnc[0]

  // st_new1 = prev_top * levIns * (1-is0)*fnc[0] * xor   +  prev_bot*xor =
  //    = + prev_top * levIns *       fnc[0] * xor     = aux2     * xor
  //      - prev_top * levIns * is0 * fnc[0] * xor     = st_old0  * xor
  //      + prev_bot *                         xor     = prev_bot * xor

  st_new1 <== (aux2 - st_old0 + prev_bot)*xor;


  // st_bot = prev_top * levIns * (1-is0)*fnc[0] * (1-xor) + prev_bot*(1-xor);
  //    = + prev_top * levIns *       fnc[0]
  //      - prev_top * levIns * is0 * fnc[0]
  //      - prev_top * levIns *       fnc[0] * xor
  //      + prev_top * levIns * is0 * fnc[0] * xor
  //      + prev_bot
  //      - prev_bot *                         xor

  st_bot <== (1-xor) * (aux2 - st_old0 + prev_bot);


  // st_upd = prev_top * (1-fnc[0]) *levIns;
  //    = + prev_top * levIns
  //      - prev_top * levIns * fnc[0]

  st_upd <== aux1 - aux2;

  // st_na = prev_new1 + prev_old0 + prev_na + prev_upd;
  //    = + prev_new1
  //      + prev_old0
  //      + prev_na
  //      + prev_upd

  st_na <== prev_new1 + prev_old0 + prev_na + prev_upd;

}
template SMTVerifierSM() {
    signal input is0;
    signal input levIns;
    signal input fnc;

    signal input prev_top;
    signal input prev_i0;
    signal input prev_iold;
    signal input prev_inew;
    signal input prev_na;

    signal output st_top;
    signal output st_i0;
    signal output st_iold;
    signal output st_inew;
    signal output st_na;

    signal prev_top_lev_ins;
    signal prev_top_lev_ins_fnc;

    prev_top_lev_ins <== prev_top * levIns;
    prev_top_lev_ins_fnc <== prev_top_lev_ins*fnc;  // prev_top * levIns * fnc

    // st_top = prev_top * (1-levIns)
    //    = + prev_top
    //      - prev_top * levIns
    st_top <== prev_top - prev_top_lev_ins;

    // st_inew = prev_top * levIns * (1-fnc)
    //   = + prev_top * levIns
    //     - prev_top * levIns * fnc
    st_inew <== prev_top_lev_ins - prev_top_lev_ins_fnc;

    // st_iold = prev_top * levIns * (1-is0)*fnc
    //   = + prev_top * levIns * fnc
    //     - prev_top * levIns * fnc * is0
    st_iold <== prev_top_lev_ins_fnc * (1 - is0);

    // st_i0 = prev_top * levIns * is0
    //  = + prev_top * levIns * is0
    st_i0 <== prev_top_lev_ins * is0;

    st_na <== prev_na + prev_inew + prev_iold + prev_i0;
}

template SMTProcessor(nLevels) {
    signal input oldRoot;
    signal output newRoot;
    signal input siblings[nLevels];
    signal input oldKey;
    signal input oldValue;
    signal input isOld0;
    signal input newKey;
    signal input newValue;
    signal input fnc[2];

    signal enabled;

    enabled <== fnc[0] + fnc[1] - fnc[0]*fnc[1];

    component hash1Old = SMTHash1();
    hash1Old.key <== oldKey;
    hash1Old.value <== oldValue;

    component hash1New = SMTHash1();
    hash1New.key <== newKey;
    hash1New.value <== newValue;

    component n2bOld = Num2Bits_strict();
    component n2bNew = Num2Bits_strict();

    n2bOld.in <== oldKey;
    n2bNew.in <== newKey;

    component smtLevIns = SMTLevIns(nLevels);
    for (var i=0; i<nLevels; i+=1) {
        smtLevIns.siblings[i] <== siblings[i];
    }
    smtLevIns.enabled <== enabled;

    component xors[nLevels];
    for (var i=0; i<nLevels; i+=1) {
        xors[i] = XOR();
        xors[i].a <== n2bOld.out[i];
        xors[i].b <== n2bNew.out[i];
    }

    component sm[nLevels];
    for (var i=0; i<nLevels; i+=1) {
        sm[i] = SMTProcessorSM();
        if (i==0) {
            sm[i].prev_top <== enabled;
            sm[i].prev_old0 <== 0;
            sm[i].prev_bot <== 0;
            sm[i].prev_new1 <== 0;
            sm[i].prev_na <== 1-enabled;
            sm[i].prev_upd <== 0;
        } else {
            sm[i].prev_top <== sm[i-1].st_top;
            sm[i].prev_old0 <== sm[i-1].st_old0;
            sm[i].prev_bot <== sm[i-1].st_bot;
            sm[i].prev_new1 <== sm[i-1].st_new1;
            sm[i].prev_na <== sm[i-1].st_na;
            sm[i].prev_upd <== sm[i-1].st_upd;
        }
        sm[i].is0 <== isOld0;
        sm[i].xor <== xors[i].out;
        sm[i].fnc[0] <== fnc[0];
        sm[i].fnc[1] <== fnc[1];
        sm[i].levIns <== smtLevIns.levIns[i];
    }
    sm[nLevels-1].st_na + sm[nLevels-1].st_new1 + sm[nLevels-1].st_old0 +sm[nLevels-1].st_upd === 1;

    component levels[nLevels];
    for (var i=nLevels-1; i != -1; i-=1) {
        levels[i] = SMTProcessorLevel();

        levels[i].st_top <== sm[i].st_top;
        levels[i].st_old0 <== sm[i].st_old0;
        levels[i].st_bot <== sm[i].st_bot;
        levels[i].st_new1 <== sm[i].st_new1;
        levels[i].st_na <== sm[i].st_na;
        levels[i].st_upd <== sm[i].st_upd;

        levels[i].sibling <== siblings[i];
        levels[i].old1leaf <== hash1Old.out;
        levels[i].new1leaf <== hash1New.out;

        levels[i].newlrbit <== n2bNew.out[i];
        if (i==nLevels-1) {
            levels[i].oldChild <== 0;
            levels[i].newChild <== 0;
        } else {
            levels[i].oldChild <== levels[i+1].oldRoot;
            levels[i].newChild <== levels[i+1].newRoot;
        }
    }

    component topSwitcher = Switcher();

    topSwitcher.sel <== fnc[0]*fnc[1];
    topSwitcher.L <== levels[0].oldRoot;
    topSwitcher.R <== levels[0].newRoot;

    component checkOldInput = ForceEqualIfEnabled();
    checkOldInput.enabled <== enabled;
    checkOldInput.in[0] <== oldRoot;
    checkOldInput.in[1] <== topSwitcher.outL;

    newRoot <== enabled * (topSwitcher.outR - oldRoot) + oldRoot;

//    topSwitcher.outL === oldRoot*enabled;
//    topSwitcher.outR === newRoot*enabled;

    // Ckeck keys are equal if updating
    component areKeyEquals = IsEqual();
    areKeyEquals.in[0] <== oldKey;
    areKeyEquals.in[1] <== newKey;

    component keysOk = MultiAND(3);
    keysOk.in[0] <== 1-fnc[0];
    keysOk.in[1] <== fnc[1];
    keysOk.in[2] <== 1-areKeyEquals.out;

    keysOk.out === 0;
}

template SMTProcessorLevel() {
    signal input st_top;
    signal input st_old0;
    signal input st_bot;
    signal input st_new1;
    signal input st_na;
    signal input st_upd;

    signal output oldRoot;
    signal output newRoot;
    signal input sibling;
    signal input old1leaf;
    signal input new1leaf;
    signal input newlrbit;
    signal input oldChild;
    signal input newChild;

    signal aux[4];

    component oldProofHash = SMTHash2();
    component newProofHash = SMTHash2();

    component oldSwitcher = Switcher();
    component newSwitcher = Switcher();

    // Old side

    oldSwitcher.L <== oldChild;
    oldSwitcher.R <== sibling;

    oldSwitcher.sel <== newlrbit;
    oldProofHash.L <== oldSwitcher.outL;
    oldProofHash.R <== oldSwitcher.outR;

    aux[0] <== old1leaf * (st_bot + st_new1 + st_upd);
    oldRoot <== aux[0] +  oldProofHash.out * st_top;

    // New side

    aux[1] <== newChild * ( st_top + st_bot);
    newSwitcher.L <== aux[1] + new1leaf*st_new1;

    aux[2] <== sibling*st_top;
    newSwitcher.R <== aux[2] + old1leaf*st_new1;

    newSwitcher.sel <== newlrbit;
    newProofHash.L <== newSwitcher.outL;
    newProofHash.R <== newSwitcher.outR;

    aux[3] <== newProofHash.out * (st_top + st_bot + st_new1);
    newRoot <==  aux[3] + new1leaf * (st_old0 + st_upd);
}

template SMTLevIns(nLevels) {
    signal input enabled;
    signal input siblings[nLevels];
    signal output levIns[nLevels];
    signal done[nLevels-1];        // Indicates if the insLevel has aready been detected.

    component isZero[nLevels];

    for (var i=0; i<nLevels; i+=1) {
        isZero[i] = IsZero();
        isZero[i].in <== siblings[i];
    }

    // The last level must always have a sibling of 0. If not, then it cannot be inserted.
    (isZero[nLevels-1].out - 1) * enabled === 0;

    levIns[nLevels-1] <== (1-isZero[nLevels-2].out);
    done[nLevels-2] <== levIns[nLevels-1];

    for (var i=nLevels-2; i>0; i-=1) {
        levIns[i] <== (1-done[i])*(1-isZero[i-1].out);
        done[i-1] <== levIns[i] + done[i];
    }
    levIns[0] <== (1-done[0]);
}

template SMTVerifier(nLevels) {
    signal input enabled;
    signal input root;
    signal input siblings[nLevels];
    signal input oldKey;
    signal input oldValue;
    signal input isOld0;
    signal input key;
    signal input value;
    signal input fnc;

    component hash1Old = SMTHash1();
    hash1Old.key <== oldKey;
    hash1Old.value <== oldValue;

    component hash1New = SMTHash1();
    hash1New.key <== key;
    hash1New.value <== value;

    component n2bOld = Num2Bits_strict();
    component n2bNew = Num2Bits_strict();

    n2bOld.in <== oldKey;
    n2bNew.in <== key;

    component smtLevIns = SMTLevIns(nLevels);
    for (var i=0; i<nLevels; i+=1) {
        smtLevIns.siblings[i] <== siblings[i];
    }
    smtLevIns.enabled <== enabled;

    component sm[nLevels];
    for (var i=0; i<nLevels; i+=1) {
        sm[i] = SMTVerifierSM();
        if (i==0) {
            sm[i].prev_top <== enabled;
            sm[i].prev_i0 <== 0;
            sm[i].prev_inew <== 0;
            sm[i].prev_iold <== 0;
            sm[i].prev_na <== 1-enabled;
        } else {
            sm[i].prev_top <== sm[i-1].st_top;
            sm[i].prev_i0 <== sm[i-1].st_i0;
            sm[i].prev_inew <== sm[i-1].st_inew;
            sm[i].prev_iold <== sm[i-1].st_iold;
            sm[i].prev_na <== sm[i-1].st_na;
        }
        sm[i].is0 <== isOld0;
        sm[i].fnc <== fnc;
        sm[i].levIns <== smtLevIns.levIns[i];
    }
    sm[nLevels-1].st_na + sm[nLevels-1].st_iold + sm[nLevels-1].st_inew + sm[nLevels-1].st_i0 === 1;

    component levels[nLevels];
    for (var i=nLevels-1; i != -1; i-=1) {
        levels[i] = SMTVerifierLevel();

        levels[i].st_top <== sm[i].st_top;
        levels[i].st_i0 <== sm[i].st_i0;
        levels[i].st_inew <== sm[i].st_inew;
        levels[i].st_iold <== sm[i].st_iold;
        levels[i].st_na <== sm[i].st_na;

        levels[i].sibling <== siblings[i];
        levels[i].old1leaf <== hash1Old.out;
        levels[i].new1leaf <== hash1New.out;

        levels[i].lrbit <== n2bNew.out[i];
        if (i==nLevels-1) {
            levels[i].child <== 0;
        } else {
            levels[i].child <== levels[i+1].root;
        }
    }


    // Check that if checking for non inclussuin and isOld0==0 then key!=old
    component areKeyEquals = IsEqual();
    areKeyEquals.in[0] <== oldKey;
    areKeyEquals.in[1] <== key;

    component keysOk = MultiAND(4);
    keysOk.in[0] <== fnc;
    keysOk.in[1] <== 1-isOld0;
    keysOk.in[2] <== areKeyEquals.out;
    keysOk.in[3] <== enabled;

    keysOk.out === 0;

    // Check the root
    component checkRoot = ForceEqualIfEnabled();
    checkRoot.enabled <== enabled;
    checkRoot.in[0] <== levels[0].root;
    checkRoot.in[1] <== root;

    // levels[0].root === root;

}

template SMTVerifierLevel() {
    signal input st_top;
    signal input st_i0;
    signal input st_iold;
    signal input st_inew;
    signal input st_na;

    signal output root;
    signal input sibling;
    signal input old1leaf;
    signal input new1leaf;
    signal input lrbit;
    signal input child;

    signal aux[2];

    component proofHash = SMTHash2();
    component switcher = Switcher();

    switcher.L <== child;
    switcher.R <== sibling;

    switcher.sel <== lrbit;
    proofHash.L <== switcher.outL;
    proofHash.R <== switcher.outR;

    aux[0] <== proofHash.out * st_top;
    aux[1] <== old1leaf*st_iold;

    root <== aux[0] + aux[1] + new1leaf*st_inew;
}

template SMTHash1() {
    signal input key;
    signal input value;
    signal output out;

    component h1 = MiMC7(91);   // Constant
    h1.x_in <== 15021630795539610737508582392395901278341266317943626182700664337106830745361;
    h1.k <== 1;

    component h2 = MiMC7(91);
    h2.x_in <== h1.out;
    h2.k <== key;

    component h3 = MiMC7(91);
    h3.x_in <== h2.out;
    h3.k <== value;

    out <== h3.out;
}

/*
    This component is used to create the 2 nodes.

    Hash2 = H(Hl | Hr)
 */

template SMTHash2() {
    signal input L;
    signal input R;
    signal output out;

    component h1 = MiMC7(91);
    h1.x_in <== 15021630795539610737508582392395901278341266317943626182700664337106830745361;
    h1.k <== L;

    component h2 = MiMC7(91);
    h2.x_in <== h1.out;
    h2.k <== R;

    out <== h2.out;
}

template MiMC7(nrounds) {
    signal input x_in;
    signal input k;
    signal output out;

    var c = [
        0,
        20888961410941983456478427210666206549300505294776164667214940546594746570981,
        15265126113435022738560151911929040668591755459209400716467504685752745317193,
        8334177627492981984476504167502758309043212251641796197711684499645635709656,
        1374324219480165500871639364801692115397519265181803854177629327624133579404,
        11442588683664344394633565859260176446561886575962616332903193988751292992472,
        2558901189096558760448896669327086721003508630712968559048179091037845349145,
        11189978595292752354820141775598510151189959177917284797737745690127318076389,
        3262966573163560839685415914157855077211340576201936620532175028036746741754,
        17029914891543225301403832095880481731551830725367286980611178737703889171730,
        4614037031668406927330683909387957156531244689520944789503628527855167665518,
        19647356996769918391113967168615123299113119185942498194367262335168397100658,
        5040699236106090655289931820723926657076483236860546282406111821875672148900,
        2632385916954580941368956176626336146806721642583847728103570779270161510514,
        17691411851977575435597871505860208507285462834710151833948561098560743654671,
        11482807709115676646560379017491661435505951727793345550942389701970904563183,
        8360838254132998143349158726141014535383109403565779450210746881879715734773,
        12663821244032248511491386323242575231591777785787269938928497649288048289525,
        3067001377342968891237590775929219083706800062321980129409398033259904188058,
        8536471869378957766675292398190944925664113548202769136103887479787957959589,
        19825444354178182240559170937204690272111734703605805530888940813160705385792,
        16703465144013840124940690347975638755097486902749048533167980887413919317592,
        13061236261277650370863439564453267964462486225679643020432589226741411380501,
        10864774797625152707517901967943775867717907803542223029967000416969007792571,
        10035653564014594269791753415727486340557376923045841607746250017541686319774,
        3446968588058668564420958894889124905706353937375068998436129414772610003289,
        4653317306466493184743870159523234588955994456998076243468148492375236846006,
        8486711143589723036499933521576871883500223198263343024003617825616410932026,
        250710584458582618659378487568129931785810765264752039738223488321597070280,
        2104159799604932521291371026105311735948154964200596636974609406977292675173,
        16313562605837709339799839901240652934758303521543693857533755376563489378839,
        6032365105133504724925793806318578936233045029919447519826248813478479197288,
        14025118133847866722315446277964222215118620050302054655768867040006542798474,
        7400123822125662712777833064081316757896757785777291653271747396958201309118,
        1744432620323851751204287974553233986555641872755053103823939564833813704825,
        8316378125659383262515151597439205374263247719876250938893842106722210729522,
        6739722627047123650704294650168547689199576889424317598327664349670094847386,
        21211457866117465531949733809706514799713333930924902519246949506964470524162,
        13718112532745211817410303291774369209520657938741992779396229864894885156527,
        5264534817993325015357427094323255342713527811596856940387954546330728068658,
        18884137497114307927425084003812022333609937761793387700010402412840002189451,
        5148596049900083984813839872929010525572543381981952060869301611018636120248,
        19799686398774806587970184652860783461860993790013219899147141137827718662674,
        19240878651604412704364448729659032944342952609050243268894572835672205984837,
        10546185249390392695582524554167530669949955276893453512788278945742408153192,
        5507959600969845538113649209272736011390582494851145043668969080335346810411,
        18177751737739153338153217698774510185696788019377850245260475034576050820091,
        19603444733183990109492724100282114612026332366576932662794133334264283907557,
        10548274686824425401349248282213580046351514091431715597441736281987273193140,
        1823201861560942974198127384034483127920205835821334101215923769688644479957,
        11867589662193422187545516240823411225342068709600734253659804646934346124945,
        18718569356736340558616379408444812528964066420519677106145092918482774343613,
        10530777752259630125564678480897857853807637120039176813174150229243735996839,
        20486583726592018813337145844457018474256372770211860618687961310422228379031,
        12690713110714036569415168795200156516217175005650145422920562694422306200486,
        17386427286863519095301372413760745749282643730629659997153085139065756667205,
        2216432659854733047132347621569505613620980842043977268828076165669557467682,
        6309765381643925252238633914530877025934201680691496500372265330505506717193,
        20806323192073945401862788605803131761175139076694468214027227878952047793390,
        4037040458505567977365391535756875199663510397600316887746139396052445718861,
        19948974083684238245321361840704327952464170097132407924861169241740046562673,
        845322671528508199439318170916419179535949348988022948153107378280175750024,
        16222384601744433420585982239113457177459602187868460608565289920306145389382,
        10232118865851112229330353999139005145127746617219324244541194256766741433339,
        6699067738555349409504843460654299019000594109597429103342076743347235369120,
        6220784880752427143725783746407285094967584864656399181815603544365010379208,
        6129250029437675212264306655559561251995722990149771051304736001195288083309,
        10773245783118750721454994239248013870822765715268323522295722350908043393604,
        4490242021765793917495398271905043433053432245571325177153467194570741607167,
        19596995117319480189066041930051006586888908165330319666010398892494684778526,
        837850695495734270707668553360118467905109360511302468085569220634750561083,
        11803922811376367215191737026157445294481406304781326649717082177394185903907,
        10201298324909697255105265958780781450978049256931478989759448189112393506592,
        13564695482314888817576351063608519127702411536552857463682060761575100923924,
        9262808208636973454201420823766139682381973240743541030659775288508921362724,
        173271062536305557219323722062711383294158572562695717740068656098441040230,
        18120430890549410286417591505529104700901943324772175772035648111937818237369,
        20484495168135072493552514219686101965206843697794133766912991150184337935627,
        19155651295705203459475805213866664350848604323501251939850063308319753686505,
        11971299749478202793661982361798418342615500543489781306376058267926437157297,
        18285310723116790056148596536349375622245669010373674803854111592441823052978,
        7069216248902547653615508023941692395371990416048967468982099270925308100727,
        6465151453746412132599596984628739550147379072443683076388208843341824127379,
        16143532858389170960690347742477978826830511669766530042104134302796355145785,
        19362583304414853660976404410208489566967618125972377176980367224623492419647,
        1702213613534733786921602839210290505213503664731919006932367875629005980493,
        10781825404476535814285389902565833897646945212027592373510689209734812292327,
        4212716923652881254737947578600828255798948993302968210248673545442808456151,
        7594017890037021425366623750593200398174488805473151513558919864633711506220,
        18979889247746272055963929241596362599320706910852082477600815822482192194401,
        13602139229813231349386885113156901793661719180900395818909719758150455500533
    ];

    var t;
    signal t2[nrounds];
    signal t4[nrounds];
    signal t6[nrounds];
    signal t7[nrounds-1];

    for (var i=0; i<nrounds; i+=1) {
        if (i==0) {
            t = k+x_in;
        } else {
            t = k + t7[i-1] + c[i];
        }
        t2[i] <== t*t;
        t4[i] <== t2[i]*t2[i];
        t6[i] <== t4[i]*t2[i];
        if (i<nrounds-1) {
            t7[i] <== t6[i]*t;
        } else {
            out <== t6[i]*t + k;
        }
    }
}

template MultiMiMC7(nInputs, nRounds) {
    signal input in[nInputs];
    signal output out;

    component mims[nInputs];
    for (var i=0; i<nInputs; i+=1) {
        mims[i] = MiMC7(nRounds);
        if (i==0) {
            mims[i].x_in <== 15021630795539610737508582392395901278341266317943626182700664337106830745361;
        } else {
            mims[i].x_in <== mims[i-1].out;
        }
        mims[i].k <== in[i];
    }

    out <== mims[nInputs-1].out;
}

template Switcher() {
    signal input sel;
    signal input L;
    signal input R;
    signal output outL;
    signal output outR;

    signal aux;

    aux <== (R-L)*sel;    // We create aux in order to have only one multiplication
    outL <==  aux + L;
    outR <== -aux + R;
}

function pointAdd(x1,y1,x2,y2) {
    var a = 168700;
    var d = 168696;

    var res[2];
    res[0] = (x1*y2 + y1*x2) / (1 + d*x1*x2*y1*y2);
    res[1] = (y1*y2 - a*x1*x2) / (1 - d*x1*x2*y1*y2);
    return res;
}

template EscalarMulW4Table(base, k) {
    signal output out[16][2];

    var i;
    var p[2];

    var dbl = base;

    for (i=0; i<k*4; i+=1) {
        dbl = pointAdd(dbl[0], dbl[1], dbl[0], dbl[1]);
    }

    out[0][0] <== 0;
    out[0][1] <== 1;
    for (i=1; i<16; i+=1) {
        p = pointAdd(out[i-1][0], out[i-1][1], dbl[0], dbl[1]);
        out[i][0] <== p[0];
        out[i][1] <== p[1];
    }
}

template FranchiseProof(nLevels) {

    signal         input censusRoot;
    signal private input censusSiblings[nLevels];
    signal private input censusIdx;

    signal private input voteSigS;
    signal private input voteSigR8x;
    signal private input voteSigR8y;

    signal         input voteValue;

    signal private input privateKey;
    
    signal         input votingId;
    signal         input nullifier;

    // -- extract public key -------------------------------------------

    component pbk = BabyPbk();
    pbk.in <== privateKey;

    // -- verify vote signature  ---------------------------------------

    component sigVerification = EdDSAMiMCVerifier();
    sigVerification.enabled <== 1;

    // signer public key (extract from private key)
    sigVerification.Ax <== pbk.Ax;
    sigVerification.Ay <== pbk.Ay;

    // signature (coordinates)
    sigVerification.S <== voteSigS;
    sigVerification.R8x <== voteSigR8x;
    sigVerification.R8y <== voteSigR8y;

    // message
    sigVerification.M <== voteValue;

    // -- verify public key is in census merkle tree ---------------------
    
    component smtCensusInclusion = SMTVerifier(nLevels);
    smtCensusInclusion.enabled <== 1;

    // check for inclusion (0 => VERIFY INCLUSION, 1=>VERIFY EXCLUSION)
    smtCensusInclusion.fnc <== 0;

    // *old* parameters are not used (only works for EXCLUSION case)
    smtCensusInclusion.oldKey <== 0;
    smtCensusInclusion.oldValue <== 0;
    smtCensusInclusion.isOld0 <== 0;

    // root and siblings
    smtCensusInclusion.root <== censusRoot;
    for (var i=0; i<nLevels; i+=1) {
        smtCensusInclusion.siblings[i] <==  censusSiblings[i];
    }

    // key and value 
    smtCensusInclusion.key <== censusIdx;

    component hashAxAy = MultiMiMC7(2, 91);
    hashAxAy.in[0] <== pbk.Ax;
    hashAxAy.in[1] <== pbk.Ay;
    smtCensusInclusion.value <== hashAxAy.out;

    // -- verify nullifier integrity -----------------------------------
    component hashPvkVid = MultiMiMC7(2, 91);
    hashPvkVid.in[0] <== privateKey;
    hashPvkVid.in[1] <== votingId ;
    nullifier === hashPvkVid.out;
}

component main = FranchiseProof(20);

    """;
