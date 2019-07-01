include "gates.circom";
include "bitify.circom";
include "comparators.circom";
include "switcher.circom";
include "smtlevins.circom";
include "smtprocessorlevel.circom";
include "smtprocessorsm.circom";
include "smthash.circom";

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

#[test] 
template test_smt_insert_blank() {
    component main =SMTProcessor(10);
    #[w]{
        main.fnc[0] <== 1;
        main.fnc[1] <== 0;
        main.oldRoot <== 0;
        main.siblings[0] <== 0;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 0;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 0;
        main.oldValue <== 0;
        main.isOld0 <== 1;
        main.newKey <== 111;
        main.newValue <== 222;
        main.newRoot === 0x247244ce4eb53753feb22877839b59c7665ac3702db1e9ea39b23fe927d42ade;
    }
}

#[test] 
template test_smt_add_another_element() {
    component main =SMTProcessor(10);
    #[w]{
        main.fnc[0] <== 1;
        main.fnc[1] <== 0;
        main.oldRoot <== 0x247244ce4eb53753feb22877839b59c7665ac3702db1e9ea39b23fe927d42ade;
        main.siblings[0] <== 0;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 0;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 111;
        main.oldValue <== 222;
        main.isOld0 <== 0;
        main.newKey <== 333;
        main.newValue <== 444;
        main.newRoot === 0x14e91e8670e2e4c83fac62ce49daddbc584bbc2f494bb099124db16c9869ca84;
    }
}

#[test] 
template test_smt_remove_element() {
    component main =SMTProcessor(10);
    #[w]{
        main.fnc[0] <== 1;
        main.fnc[1] <== 1;
        main.oldRoot <== 0x14e91e8670e2e4c83fac62ce49daddbc584bbc2f494bb099124db16c9869ca84;
        main.siblings[0] <== 0;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 0;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 333;
        main.oldValue <== 444;
        main.isOld0 <== 0;
        main.newKey <== 111;
        main.newValue <== 222;
        main.newRoot === 0x2d97772416a8cea7f9161c59f08076113ef638885ad0441570355f7b74a368dc;
    }
}

#[test] 
template test_smt_remove_another_element() {
    component main =SMTProcessor(10);
    #[w]{
        main.fnc[0] <== 1;
        main.fnc[1] <== 1;
        main.oldRoot <== 0x2d97772416a8cea7f9161c59f08076113ef638885ad0441570355f7b74a368dc;
        main.siblings[0] <== 0;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 0;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 0;
        main.oldValue <== 0;
        main.isOld0 <== 1;
        main.newKey <== 333;
        main.newValue <== 444;
        main.newRoot === 0x0;
    }
}

#[test] 
template test_smt_update() {
    component main =SMTProcessor(10);
    #[w]{
        main.fnc[0] <== 0;
        main.fnc[1] <== 1;
        main.oldRoot <== 7144490948648913323643490225720764606754398422608274858868522414539653898462;
        main.siblings[0] <== 5308339863289897018477020694643060162563592147764909710270574730002055605778;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 12633975236947324549554904811138904210334136753206440087462433895022867255191;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 32;
        main.oldValue <== 3232;
        main.isOld0 <== 0;
        main.newKey <== 32;
        main.newValue <== 323232;
        main.newRoot === 0x13fc6f9c4bcf2a4f8191d7693996bf155a098243fd8825308d436acb796dda18;
    }
}

#[test] 
template test_smt_insert_claim() {
    component main =SMTProcessor(10);
    #[w] {
        main.fnc[0] <== 1;
        main.fnc[1] <== 0;
        main.oldRoot <== 0;
        main.siblings[0] <== 0;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 0;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 0;
        main.oldValue <== 0;
        main.isOld0 <== 1;
        main.newKey <== 0x89fd2edc0a6dd763b006c0a1903b09fcb3b51aabfff7a54ffb51ce940b8933f;
        main.newValue <== 0x24ae9775f16de9b0cdca722df4d6678c08a817bd40ed4575ef46a375a07daf79;
        main.newRoot === 0x2e190f638990104298cc690fa9040e805c3f1fbcf9a219d89e8f7c68fa8fbdec;
    }
}