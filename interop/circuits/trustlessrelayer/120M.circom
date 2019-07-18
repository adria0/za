include "claimsrootupdate.circom";

template TheBigOne(N,nLevelsUser,nLevelsRelayer) {

    signal input oldRelayerRoot[N];
    signal input newRelayerRoot[N];
    signal input oldUserRoot[N];
    signal input idIdentity[N];
    signal input era[N];
    signal input newUserRoot[N];
    signal input newUserRootVersion[N];
    signal input sigKeyX[N];
    signal input sigKeyY[N];
    signal input sigS[N];
    signal input sigR8x[N];
    signal input sigR8y[N];
    signal input signingKeyInclussion_siblings[N][nLevelsUser];
    signal input signingKeyExclusion_siblings[N][nLevelsUser];
    signal input signingKeyExclusion_oldKey[N];
    signal input signingKeyExclusion_oldValue[N];
    signal input signingKeyExclusion_isOld0[N];
    signal input oldRootInclusion_siblings[N][nLevelsRelayer];
    signal input relayerInsert_siblings[N][nLevelsRelayer];
    signal input relayerInsert_oldKey[N];
    signal input relayerInsert_oldValue[N];
    signal input relayerInsert_isOld0[N];

    component CRU[N];
    
    for (var n=0;n<N;n+=1) {
        CRU[n] = ClaimRootUpdate(nLevelsUser,nLevelsRelayer);
       
        CRU[n].oldRelayerRoot <== oldRelayerRoot[n];
        CRU[n].newRelayerRoot <== newRelayerRoot[n];
        CRU[n].oldUserRoot <== oldUserRoot[n];
        CRU[n].idIdentity <== idIdentity[n];
        CRU[n].era <== era[n];
        CRU[n].newUserRoot <== newUserRoot[n];
        CRU[n].newUserRootVersion <== newUserRootVersion[n];
        CRU[n].sigKeyX <== sigKeyX[n];
        CRU[n].sigKeyY <== sigKeyY[n];
        CRU[n].sigS <== sigS[n];
        CRU[n].sigR8x <== sigR8x[n];
        CRU[n].sigR8y <== sigR8y[n];
        
        for (var lu=0;lu<nLevelsUser;lu+=1) {
            CRU[n].signingKeyInclussion_siblings[lu] <== signingKeyInclussion_siblings[n][lu];
            CRU[n].signingKeyExclusion_siblings[lu] <== signingKeyExclusion_siblings[n][lu];
        }
        
        CRU[n].signingKeyExclusion_oldKey <== signingKeyExclusion_oldKey[n];
        CRU[n].signingKeyExclusion_oldValue <== signingKeyExclusion_oldValue[n];
        CRU[n].signingKeyExclusion_isOld0 <== signingKeyExclusion_isOld0[n];

        for (var lr=0;lr<nLevelsRelayer;lr+=1) {
            CRU[n].oldRootInclusion_siblings[lr] <== oldRootInclusion_siblings[n][lr];
            CRU[n].relayerInsert_siblings[lr] <== relayerInsert_siblings[n][lr];
        }

        CRU[n].relayerInsert_oldKey <== relayerInsert_oldKey[n];
        CRU[n].relayerInsert_oldValue <== relayerInsert_oldValue[n];
        CRU[n].relayerInsert_isOld0 <== relayerInsert_isOld0[n];

        if (n > 0) {
            CRU[n].oldRelayerRoot === CRU[n-1].newRelayerRoot;
        }

    }

}

component main = TheBigOne(240,140,140);
