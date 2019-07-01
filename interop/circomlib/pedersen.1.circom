
template Num2Bits() {
    signal input in;
    signal output out;
    out <== in;
}

template Segment() {
    signal input in;
    signal input base;
    signal output out;

    out <== in+base;

}

template Pedersen() {
    signal input in;
    signal output out;

    component segments = Segment();
    segments.base <== 2;
    segments.in <== in;

    out <== segments.out;

}

template pedersen256_helper() {
    signal input in;
    signal output out;

    component pedersen = Pedersen();

    component n2b;
    n2b = Num2Bits();

    in ==> n2b.in;
    
    pedersen.in <== n2b.out;
    pedersen.out ==> out;
}


#[test]
template t() {
    component main =pedersen256_helper();
    #[w] {
        main.in <== 0;
    }
}
