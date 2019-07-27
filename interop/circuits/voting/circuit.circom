include "../circomlib/circuits/mimc.circom";
include "../circomlib/circuits/bitify.circom";
include "../circomlib/circuits/escalarmulfix.circom";
include "../circomlib/circuits/eddsamimc.circom";
include "../circomlib/circuits/smt/smtverifier.circom";
include "../circomlib/circuits/smt/smtprocessor.circom";

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

#[test]
template test_voting_20() {
    component main = FranchiseProof(20);

    #[w] {
        main.privateKey <== 3876493977147089964395646989418653640709890493868463039177063670701706079087;
        main.votingId <== 1;
        main.nullifier <== 3642967737206797788953266792789642811467066441527044263456672813575084154491;
        main.censusRoot <== 19335063285569462410966731541261274523671078966610109902395268519183816138000;
        for (var n=0;n<20;n+=1) {
            main.censusSiblings[n] <== 0;
        }
        main.censusIdx <== 1337;
        main.voteSigS <== 1506558731080100151400643495683521762973142364485982380016443632063521613779;
        main.voteSigR8x <== 18137411472623093392316389329188709228585113201501107240811900197785235422996;
        main.voteSigR8y <== 3319823053651665777987773445125343092037295151949542813138094827788048737351;
        main.voteValue <== 2;
    }
}

component main = FranchiseProof(20);
