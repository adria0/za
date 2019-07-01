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

/*

SMTVerifier is a component to verify inclusion/exclusion of an element in the tree


fnc:  0 -> VERIFY INCLUSION
      1 -> VERIFY NOT INCLUSION

 */


include "gates.circom";
include "bitify.circom";
include "comparators.circom";
include "switcher.circom";
include "smtlevins.circom";
include "smtverifierlevel.circom";
include "smtverifiersm.circom";
include "smthash.circom";

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

#[test] 
template test_smt_inclusion() {
    component main =SMTVerifier(10);
    #[w]{
        main.enabled <== 1;
        main.fnc <== 0;
        main.root <== 11551450462129540273398008002958622457574029324157946159527568407658859887829;
        main.siblings[0] <== 13455034059297281016272585298528306605866068217171776345671286666756502579091;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 1861165906443293922235287625340159227217462493991641436708152769502026456756;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 32;
        main.oldValue <== 3232;
        main.isOld0 <== 0;
        main.key <== 32;
        main.value <== 3232;
    }
}

#[test] 
template test_smt_exclusion() {
    component main =SMTVerifier(10);
    #[w]{
        main.enabled <== 1;
        main.fnc <== 1;
        main.root <== 11551450462129540273398008002958622457574029324157946159527568407658859887829;
        main.siblings[0] <== 13455034059297281016272585298528306605866068217171776345671286666756502579091;
        main.siblings[1] <== 0;
        main.siblings[2] <== 0;
        main.siblings[3] <== 1861165906443293922235287625340159227217462493991641436708152769502026456756;
        main.siblings[4] <== 0;
        main.siblings[5] <== 0;
        main.siblings[6] <== 0;
        main.siblings[7] <== 0;
        main.siblings[8] <== 0;
        main.siblings[9] <== 0;
        main.oldKey <== 32;
        main.oldValue <== 3232;
        main.isOld0 <== 0;
        main.key <== 64;
        main.value <== 0;
    }
}
