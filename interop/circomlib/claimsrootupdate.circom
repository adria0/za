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


include "smtverifier.circom";
include "smtprocessor.circom";
include "eddsamimc.circom";
include "comparators.circom";
include "mimc.circom";
include "sign.circom";

template BuildUserRootClaims() {
    signal input version;
    signal input idIdentity;
    signal input era;
    signal input newRroot;
    signal input oldRroot;
    signal output old_hi;
    signal output old_hv;
    signal output new_hi;
    signal output new_hv;

    var CLAIMTYPE = 2;

    component versionBits = Num2Bits(32);
    versionBits.in <== version;
    component eraBits = Num2Bits(32);
    eraBits.in <== era;

    component idx = Bits2Num(128);
    for (var i=0; i<64; i+=1) {
        idx.in[i] <== (CLAIMTYPE >> i) & 1;
    }
    for (var i=0; i<32; i+=1) {
        idx.in[64+i] <== versionBits.out[i];
    }
    for (var i=0; i<32; i+=1) {
        idx.in[96+i] <== eraBits.out[i];
    }

    component hashIdxNew = MultiMiMC7(2, 91);
    hashIdxNew.in[0] <== idIdentity;
    hashIdxNew.in[1] <== idx.out ;
    new_hi <== hashIdxNew.out;
    component hashValueNew = MultiMiMC7(2, 91);
    hashValueNew.in[0] <== 0;
    hashValueNew.in[1] <== newRroot;
    new_hv <== hashValueNew.out;

    component hashIdxOld = MultiMiMC7(2, 91);
    hashIdxOld.in[0] <== idIdentity;
    hashIdxOld.in[1] <== idx.out - 2**64; // Decrement version.  (If 0 oldRoot is not checked)
    old_hi <== hashIdxOld.out;
    component hashValueOld = MultiMiMC7(2, 91);
    hashValueOld.in[0] <== 0;
    hashValueOld.in[1] <== oldRroot;
    old_hv <== hashValueOld.out;
}

template BuildAuthorizeKeyClaims() {
    signal input Ax;
    signal input Ay;
    signal output inc_hi;
    signal output inc_hv;
    signal output exc_hi;

    var CLAIMTYPE = 1;

    component Axbin = Num2Bits_strict();
    Axbin.in <== Ax;
    component sign = Sign();
    for (var i=0; i<254; i+=1) {
        sign.in[i] <== Axbin.out[i];
    }
    component idx = Bits2Num(97);
    for (var i=0; i<64; i+=1) {
        idx.in[i] <== (CLAIMTYPE >> i) & 1;
    }
    for (var i=0; i<32; i+=1) {
        idx.in[64+i] <== 0;
    }
    idx.in[96] <== sign.sign;


    component hashIdxInc = MultiMiMC7(2, 91);
    hashIdxInc.in[0] <== Ay;
    hashIdxInc.in[1] <== idx.out;
    inc_hi <== hashIdxInc.out;

    component hashValueInc = MultiMiMC7(2, 91);
    hashValueInc.in[0] <== 0;
    hashValueInc.in[1] <== 0;
    inc_hv <== hashValueInc.out;

    component hashIdxExc = MultiMiMC7(2, 91);
    hashIdxExc.in[0] <== Ay;
    hashIdxExc.in[1] <== idx.out + 2**64; // Increment version.
    exc_hi <== hashIdxExc.out;
}

template ClaimRootUpdate(nLevelsRelayer, nLevelsUser) {
    signal input oldRelayerRoot;
    signal input newRelayerRoot;

    signal input oldUserRoot;
    signal input idIdentity;
    signal input era;
    signal input newUserRoot;
    signal input newUserRootVersion;
    signal input sigKeyX;
    signal input sigKeyY;
    signal input sigS;
    signal input sigR8x;
    signal input sigR8y;

    // Needed for inclusion verification of the key
    signal input signingKeyInclussion_siblings[nLevelsUser];

    // Needed for exclusion verification of the key (not revoked)
    signal input signingKeyExclusion_siblings[nLevelsUser];
    signal input signingKeyExclusion_oldKey;
    signal input signingKeyExclusion_oldValue;
    signal input signingKeyExclusion_isOld0;

    // Needed for old root inclusion
    signal input oldRootInclusion_siblings[nLevelsRelayer];

    // Needed for insert
    signal input relayerInsert_siblings[nLevelsRelayer];
    signal input relayerInsert_oldKey;
    signal input relayerInsert_oldValue;
    signal input relayerInsert_isOld0;

    // The Version 0 can be introduced freely by the operator;
    component verIsZero = IsZero();
    verIsZero.in <== newUserRootVersion;

    // Build User Root Claims (new and old)
    component buildUserRootClaims = BuildUserRootClaims();
    buildUserRootClaims.version <== newUserRootVersion;
    buildUserRootClaims.idIdentity <== idIdentity;
    buildUserRootClaims.era <== era;
    buildUserRootClaims.newRroot <== newUserRoot;
    buildUserRootClaims.oldRroot <== oldUserRoot;

    // Verify the signature
    component signMsgHash = MultiMiMC7(3, 91);
    signMsgHash.in[0] <== 1234123412341234; // change root method
    signMsgHash.in[1] <== buildUserRootClaims.new_hi;
    signMsgHash.in[2] <== buildUserRootClaims.new_hv;

    component sigVerification = EdDSAMiMCVerifier();
    sigVerification.enabled <== 1-verIsZero.out;
    sigVerification.R8x <== sigR8x;
    sigVerification.R8y <== sigR8y;
    sigVerification.Ax <== sigKeyX;
    sigVerification.Ay <== sigKeyY;
    sigVerification.S <== sigS;
    sigVerification.M <== signMsgHash.out;

    // Build the key authorization claims.
    component buildAuthorizeKeyClaims = BuildAuthorizeKeyClaims();
    buildAuthorizeKeyClaims.Ax <== sigKeyX;
    buildAuthorizeKeyClaims.Ay <== sigKeyY;

    // Verify that the key is in the old root
    component smtSignKeyInclusion = SMTVerifier(nLevelsUser);
    smtSignKeyInclusion.enabled <== 1-verIsZero.out;
    smtSignKeyInclusion.fnc <== 0;
    smtSignKeyInclusion.root <== oldUserRoot;
    for (var i=0; i<nLevelsUser; i+=1) {
        smtSignKeyInclusion.siblings[i] <==  signingKeyInclussion_siblings[i];
    }
    smtSignKeyInclusion.oldKey <== 0;
    smtSignKeyInclusion.oldValue <== 0;
    smtSignKeyInclusion.isOld0 <== 0;
    smtSignKeyInclusion.key <== buildAuthorizeKeyClaims.inc_hi;
    smtSignKeyInclusion.value <== buildAuthorizeKeyClaims.inc_hv;

    // Verify that the key is not revoked
    component smtSignKeyExclusion = SMTVerifier(nLevelsUser);
    smtSignKeyExclusion.enabled <== 1-verIsZero.out;
    smtSignKeyExclusion.fnc <== 1;
    smtSignKeyExclusion.root <== oldUserRoot;
    for (var i=0; i<nLevelsUser; i+=1) {
        smtSignKeyExclusion.siblings[i] <==  signingKeyExclusion_siblings[i];
    }

    smtSignKeyExclusion.oldKey <== signingKeyExclusion_oldKey;
    smtSignKeyExclusion.oldValue <== signingKeyExclusion_oldValue;
    smtSignKeyExclusion.isOld0 <== signingKeyExclusion_isOld0;
    smtSignKeyExclusion.key <== buildAuthorizeKeyClaims.exc_hi;
    smtSignKeyExclusion.value <== 0;

    // Verify that the old root is on the relayer tree
    component smtOldRootInclusion = SMTVerifier(nLevelsRelayer);
    smtOldRootInclusion.enabled <== 1-verIsZero.out;
    smtOldRootInclusion.fnc <== 0;
    smtOldRootInclusion.root <== oldRelayerRoot;
    for (var i=0; i<nLevelsRelayer; i+=1) {
        smtOldRootInclusion.siblings[i] <==  oldRootInclusion_siblings[i];
    }
    smtOldRootInclusion.oldKey <== 0;
    smtOldRootInclusion.oldValue <== 0;
    smtOldRootInclusion.isOld0 <== 0;
    smtOldRootInclusion.key <== buildUserRootClaims.old_hi;
    smtOldRootInclusion.value <== buildUserRootClaims.old_hv;

    // Process the insert
    component smtRelayerInsert = SMTProcessor(nLevelsRelayer);
    smtRelayerInsert.fnc[0] <==  1;
    smtRelayerInsert.fnc[1] <==  0;
    smtRelayerInsert.oldRoot <== oldRelayerRoot;
    for (var i=0; i<nLevelsRelayer; i+=1) {
        smtRelayerInsert.siblings[i] <==  relayerInsert_siblings[i];
    }
    smtRelayerInsert.oldKey <== relayerInsert_oldKey;
    smtRelayerInsert.oldValue <== relayerInsert_oldValue;
    smtRelayerInsert.isOld0 <== relayerInsert_isOld0;
    smtRelayerInsert.newKey <== buildUserRootClaims.new_hi;
    smtRelayerInsert.newValue <== buildUserRootClaims.new_hv;

    smtRelayerInsert.newRoot === newRelayerRoot;

}

#[test]
template test_BuildUserRootClaims() {
    component main = BuildUserRootClaims();
    #[w] {
        main.version <== 0;
        main.idIdentity <== 1234;
        main.era <== 0;
        main.newRroot <== 7149014917815960042505969439971619119991011354574443484106856202048948095881;
        main.oldRroot <== 0;

        main.old_hi === 0x182ee393cfcbf975e25af01296a1a3bb70f66049e7fff0c797f268d064e5b550;
        main.old_hv === 0x1541a6b5aa9bf7d9be3d5cb0bcc7cacbca26242016a0feebfc19c90f2224baed;
        main.new_hi === 0x89fd2edc0a6dd763b006c0a1903b09fcb3b51aabfff7a54ffb51ce940b8933f;
        main.new_hv === 0x24ae9775f16de9b0cdca722df4d6678c08a817bd40ed4575ef46a375a07daf79;
    }
}

template test_BuildAuthorizeKeyClaims() {
    component main = BuildAuthorizeKeyClaims();
    #[w] {
        main.Ax <== 2610057752638682202795145288373380503107623443963127956230801721756904484787;
        main.Ay <== 16617171478497210597712478520507818259149717466230047843969353176573634386897;
        main.inc_hi === 0x1a75f5ec4fbc824c07a75d08848b45c7dff0f264f48fddbd5fed4fb8495cb381;
        main.inc_hv === 0x1541a6b5aa9bf7d9be3d5cb0bcc7cacbca26242016a0feebfc19c90f2224baed;
        main.exc_hi === 0x2d8b2bb67bec2fce9e6be5bf251869dd5430439f4f21a934726bfad2bd884ad9;
    }
}

#[test]
template test_claimrootupdate() {
    component main = ClaimRootUpdate(10, 10);
    #[w] {
        main.oldRelayerRoot <== 0;
        main.newRelayerRoot <== 7149014917815960042505969439971619119991011354574443484106856202048948095881;
        main.oldUserRoot <== 0;
        main.idIdentity <== 1234;
        main.era <== 0;
        main.newUserRoot <== 9164435831827345487378393454304824441756195871900421654673163382659437536500;
        main.newUserRootVersion <== 0;
        main.sigKeyX <== 2610057752638682202795145288373380503107623443963127956230801721756904484787;
        main.sigKeyY <== 16617171478497210597712478520507818259149717466230047843969353176573634386897;
        main.sigS <== 1043684292397350507108850650525684376439518174906421843791686494893141984604;
        main.sigR8x <== 15395507177505103995870174907385016443674539549000225563955344755542040525521;
        main.sigR8y <== 8315800102674792436694752406326424429944804229252094505145784491665416975932;
        
        main.signingKeyExclusion_oldKey <== 0;
        main.signingKeyExclusion_oldValue <== 0;
        main.signingKeyExclusion_isOld0 <== 0;
        main.relayerInsert_oldKey <== 0;
        main.relayerInsert_oldValue <== 0;
        main.relayerInsert_isOld0 <== 1;    

        var signingKeyInclussion_siblings =  [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
        var signingKeyExclusion_siblings =  [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
        var oldRootInclusion_siblings = [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
        var relayerInsert_siblings = [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

        for (var i=0;i<10;i+=1) {
            main.signingKeyInclussion_siblings[i] <== signingKeyInclussion_siblings[i];
            main.signingKeyExclusion_siblings[i] <== signingKeyExclusion_siblings[i];
            main.oldRootInclusion_siblings[i] <== oldRootInclusion_siblings[i];
            main.relayerInsert_siblings[i] <== relayerInsert_siblings[i];
        }
    }
}

