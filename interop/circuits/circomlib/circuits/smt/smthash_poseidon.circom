include "../poseidon.circom";


/*
    Hash1 = H(1 | key | value)
 */

template SMTHash1() {
    signal input key;
    signal input value;
    signal output out;

    component h = Poseidon(3, 6, 8, 57);   // Constant
    h.inputs[0] <== key;
    h.inputs[1] <== value;
    h.inputs[2] <== 1;

    out <== h.out;
}

/*
    This component is used to create the 2 nodes.

    Hash2 = H(Hl | Hr)
 */

template SMTHash2() {
    signal input L;
    signal input R;
    signal output out;

    component h = Poseidon(2, 6, 8, 57);   // Constant
    h.inputs[0] <== L;
    h.inputs[1] <== R;

    out <== h.out;
}