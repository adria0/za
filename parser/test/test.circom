include "claimsrootupdate.circom";

template CI(n) {
    component roots[n];
    for (var i=0;i<n;i+=1) {
        dbg!(i);
        roots[i] = ClaimRootUpdate(140,140);
    }
}

component main = CI(100);

