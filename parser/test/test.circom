// include "claimsrootupdate.circom";
// component main = ClaimRootUpdate(2,2);

include "mimc.circom";
include "constants.circom";
include "babyjub.circom";

component main = MiMC7(12);
