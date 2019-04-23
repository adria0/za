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
    signal output newRelayerRoot;

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
    smtRelayerInsert.newRoot <== newRelayerRoot;
    for (var i=0; i<nLevelsRelayer; i+=1) {
        smtRelayerInsert.siblings[i] <==  relayerInsert_siblings[i];
    }
    smtRelayerInsert.oldKey <== relayerInsert_oldKey;
    smtRelayerInsert.oldValue <== relayerInsert_oldValue;
    smtRelayerInsert.isOld0 <== relayerInsert_isOld0;
    smtRelayerInsert.newKey <== buildUserRootClaims.new_hi;
    smtRelayerInsert.newValue <== buildUserRootClaims.new_hv;
}

