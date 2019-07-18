include "../circomlib/circuits/mimc.circom";
include "../circomlib/circuits/eddsamimc.circom";
include "../circomlib/circuits/bitify.circom";
include "./helpers/tx_existence_check.circom";
include "./helpers/balance_existence_check.circom";
include "./helpers/balance_leaf.circom";
include "./helpers/get_merkle_root.circom";
include "./helpers/if_gadgets.circom";


template Main(n,m) {
// n is depth of balance tree
// m is depth of transactions tree
// for each proof][update 2**m transactions

    // Merkle root of transactions tree
    signal input txRoot;

    // Merkle proof for transaction in tx tree
    signal private input paths2txRoot[2**m][m];

    // binary vector indicating whether node in tx proof is left or right
    signal private input paths2txRootPos[2**m][m];

    // Merkle root of old balance tree
    signal input currentState;

    // intermediate roots (two for each tx)][final element is last.
    signal private input intermediateRoots[2**(m+1)+1];

    // Merkle proof for sender account in balance tree
    signal private input paths2rootFrom[2**m][n];

    // binary vector indicating whether node in balance proof for sender account
    // is left or right 
    signal private input paths2rootFromPos[2**m][n];

    // Merkle proof for receiver account in balance tree
    signal private input paths2rootTo[2**m][n];

    // binary vector indicating whether node in balance proof for receiver account
    // is left or right 
    signal private input paths2rootToPos[2**m][n];
    
    // tx info][10 fields
    signal private input fromX[2**m]; //sender address x coordinate
    signal private input fromY[2**m]; //sender address y coordinate
    signal private input fromIndex[2**m]; //sender account leaf index
    signal private input toX[2**m]; // receiver address x coordinate
    signal private input toY[2**m]; // receiver address y coordinate
    signal private input nonceFrom[2**m]; // sender account nonce
    signal private input amount[2**m]; // amount being transferred
    signal private input tokenTypeFrom[2**m]; // sender token type
    signal private input R8x[2**m]; // sender signature
    signal private input R8y[2**m]; // sender signature
    signal private input S[2**m]; // sender signature

    // additional account info (not included in tx)
    signal private input balanceFrom[2**m]; // sender token balance

    signal private input balanceTo[2**m]; // receiver token balance
    signal private input nonceTo[2**m]; // receiver account nonce
    signal private input tokenTypeTo[2**m]; // receiver token type

    // // new balance tree Merkle root
    signal output out;

    var NONCE_MAX_VALUE = 100;

    // constant zero address
                         
    var ZERO_ADDRESS_X = 0;
    var ZERO_ADDRESS_Y = 0;

    component txExistence[2**m];
    component senderExistence[2**m];
    component ifBothHighForceEqual[2**m];
    component newSender[2**m];
    component computedRootFromNewSender[2**m];
    component receiverExistence[2**m];
    component newReceiver[2**m];
    component allLow[2**m];
    component ifThenElse[2**m];
    component computedRootFromNewReceiver[2**m];


    currentState === intermediateRoots[0];

    for (var i = 0; i < 2**m; i+=1) {

        // transactions existence and signature check
        txExistence[i] = TxExistence(m);
        txExistence[i].fromX <== fromX[i];
        txExistence[i].fromY <== fromY[i];
        txExistence[i].fromIndex <== fromIndex[i];
        txExistence[i].toX <== toX[i];
        txExistence[i].toY <== toY[i];
        txExistence[i].nonce <== nonceFrom[i];
        txExistence[i].amount <== amount[i];
        txExistence[i].tokenType <== tokenTypeFrom[i];

        txExistence[i].txRoot <== txRoot;

        for (var j = 0; j < m; j+=1){
            txExistence[i].paths2rootPos[j] <== paths2txRootPos[i][j] ;
            txExistence[i].paths2root[j] <== paths2txRoot[i][j];
        }

        txExistence[i].R8x <== R8x[i];
        txExistence[i].R8y <== R8y[i];
        txExistence[i].S <== S[i];
    
        // sender existence check
        senderExistence[i] = BalanceExistence(n);
        senderExistence[i].x <== fromX[i];
        senderExistence[i].y <== fromY[i];
        senderExistence[i].balance <== balanceFrom[i];
        senderExistence[i].nonce <== nonceFrom[i];
        senderExistence[i].tokenType <== tokenTypeFrom[i];

        senderExistence[i].balanceRoot <== intermediateRoots[2*i];
        for (var j = 0; j < n; j+=1){
            senderExistence[i].paths2rootPos[j] <== paths2rootFromPos[i][j];
            senderExistence[i].paths2root[j] <== paths2rootFrom[i][j];
        }
    
        // @AMB commented ------------------------------------
        // balance checks
        // balanceFrom[i] - amount[i] <= balanceFrom[i];
        // balanceTo[i] + amount[i] >= balanceTo[i];
        // nonceFrom[i] != NONCE_MAX_VALUE;
        // @AMB commented ------------------------------------

        //-----CHECK TOKEN TYPES === IF NON-WITHDRAWS-----//
        ifBothHighForceEqual[i] = IfBothHighForceEqual();
        ifBothHighForceEqual[i].check1 <== toX[i];
        ifBothHighForceEqual[i].check2 <== toY[i];
        ifBothHighForceEqual[i].a <== tokenTypeTo[i];
        ifBothHighForceEqual[i].b <== tokenTypeFrom[i];
        //-----END CHECK TOKEN TYPES-----//  

        // subtract amount from sender balance; increase sender nonce 
        newSender[i] = BalanceLeaf();
        newSender[i].x <== fromX[i];
        newSender[i].y <== fromY[i];
        newSender[i].balance <== balanceFrom[i] - amount[i];
        newSender[i].nonce <== nonceFrom[i] + 1;
        newSender[i].tokenType <== tokenTypeFrom[i];

        // get intermediate root from new sender leaf
        computedRootFromNewSender[i] = GetMerkleRoot(n);
        computedRootFromNewSender[i].leaf <== newSender[i].out;
        for (var j = 0; j < n; j+=1){
            computedRootFromNewSender[i].paths2root[j] <== paths2rootFrom[i][j];
            computedRootFromNewSender[i].paths2rootPos[j] <== paths2rootFromPos[i][j];
        }

        // check that intermediate root is consistent with input

        computedRootFromNewSender[i].out === intermediateRoots[2*i  + 1];
        //-----END SENDER IN TREE 2 AFTER DEDUCTING CHECK-----//


        // receiver existence check in intermediate root from new sender
        receiverExistence[i] = BalanceExistence(n);
        receiverExistence[i].x <== toX[i];
        receiverExistence[i].y <== toY[i];
        receiverExistence[i].balance <== balanceTo[i];
        receiverExistence[i].nonce <== nonceTo[i];
        receiverExistence[i].tokenType <== tokenTypeTo[i];

        receiverExistence[i].balanceRoot <== intermediateRoots[2*i + 1];
        for (var j = 0; j < n; j+=1){
            receiverExistence[i].paths2rootPos[j] <== paths2rootToPos[i][j] ;
            receiverExistence[i].paths2root[j] <== paths2rootTo[i][j];
        }

        //-----CHECK RECEIVER IN TREE 3 AFTER INCREMENTING-----//
        newReceiver[i] = BalanceLeaf();
        newReceiver[i].x <== toX[i];
        newReceiver[i].y <== toY[i];

        // if receiver is zero address][do not change balance
        // otherwise add amount to receiver balance
        allLow[i] = AllLow(2);
        allLow[i].in[0] <== toX[i];
        allLow[i].in[1] <== toY[i];

        ifThenElse[i] = IfAThenBElseC();
        ifThenElse[i].aCond <== allLow[i].out;
        ifThenElse[i].bBranch <== balanceTo[i];
        ifThenElse[i].cBranch <== balanceTo[i] + amount[i];  

        newReceiver[i].balance <== ifThenElse[i].out; 
        newReceiver[i].nonce <== nonceTo[i];
        newReceiver[i].tokenType <== tokenTypeTo[i];


        // get intermediate root from new receiver leaf
        computedRootFromNewReceiver[i] = GetMerkleRoot(n);
        computedRootFromNewReceiver[i].leaf <== newReceiver[i].out;
        for (var j = 0; j < n; j+=1){
            computedRootFromNewReceiver[i].paths2root[j] <== paths2rootTo[i][j];
            computedRootFromNewReceiver[i].paths2rootPos[j] <== paths2rootToPos[i][j];
        }

        // check that intermediate root is consistent with input
        computedRootFromNewReceiver[i].out === intermediateRoots[2*i  + 2];
        //-----END CHECK RECEIVER IN TREE 3 AFTER INCREMENTING-----//
    }
    out <== computedRootFromNewReceiver[2**m - 1].out;


}

#[test] 
template test_rollup_42() {

    component main = Main(4,2);

    #[w] {
    
        var txRoot  = 14053325031894235002744541221369412510941171790893507881802249870625790656164;
        var paths2txRoot = [
            [
                10606288839091048139595593807393646981074580998452498498102191274802839716974,
                2608995037327946514938120021228249424325886260519057431141160511257427858790
            ],
            [
                7098655440094613198080048525953255637313939645539008401738087356838738631323,
                2608995037327946514938120021228249424325886260519057431141160511257427858790
            ],
            [
                6492542593436611836707707094016570668509448571347639471019727884216164188730,
                7470962133072819569987354716789447159183691516375921321548127336039334673689
            ],
            [
                11541976254808148998964401650717561936567567314700455417023466748527514809732,
                7470962133072819569987354716789447159183691516375921321548127336039334673689
            ]
        ];
        var paths2txRootPos = [
            [
                0,
                0
            ],
            [
                1,
                0
            ],
            [
                0,
                1
            ],
            [
                1,
                1
            ]
        ];
        var currentState = 18648130918012499145379763689148123788934814534108898637620846058357562030290;
        var newState = 11906244236079007243914245890798662246316717426994617587947898072260792776007;
        var intermediateRoots =[
            18648130918012499145379763689148123788934814534108898637620846058357562030290,
            13984568938318779243384249857072528475334097093971526730313818037575303895156,
            7912212943187484548235253028205895169032161272943992019907208995859312919611,
            16950419554751951994045449366301225585280586440249526156204829834323753076899,
            16950419554751951994045449366301225585280586440249526156204829834323753076899,
            20041328540498103896130506725191726766126968064550434617504774768356266583517,
            16416606783193441312741953679831197136946893312976088673969971211837418012007,
            11906244236079007243914245890798662246316717426994617587947898072260792776007,
            11906244236079007243914245890798662246316717426994617587947898072260792776007
        ];
        var paths2rootFrom = [
            [
                4109936297481065161960292800159291683215893076819275861378668204678997087220,
                21258209912454469031713286570427863261191386906614357080811350774760300345338,
                12451475136974473939830535751387710903307826290572049949123529495803792686485,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                8299364841174469455405454211783579991562473885062627363827313803783551655810,
                12600434264236862446033850254343057583464824912413717048793795328328330212605,
                7219035791049455692605897676410675074070701617801223212184809210066032311734,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                19244000840215274338878635978982642861967216453201900581181802823566931416097,
                21258209912454469031713286570427863261191386906614357080811350774760300345338,
                8696772024132538335301384667037109350601038575788952566222203798816610283060,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                17400342990847699622034895903486521563192531922107760411846337521891653711537,
                14941305165750888611927396742686477481151661736272208988766205422971408648356,
                12862262498472802906442187303462198084729150273332424737168355509216970984467,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ]
        ];
        var paths2rootFromPos = [
            [
                0,
                1,
                0,
                0
            ],
            [
                0,
                0,
                1,
                0
            ],
            [
                1,
                1,
                0,
                0
            ],
            [
                1,
                0,
                0,
                0
            ]
        ];
        var paths2rootTo = [
            [
                8299364841174469455405454211783579991562473885062627363827313803783551655810,
                12600434264236862446033850254343057583464824912413717048793795328328330212605,
                7219035791049455692605897676410675074070701617801223212184809210066032311734,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                1049925881610635661188217025618392685698753415026321679317922077335053921676,
                17692171668577534793226005050113631391418282284291724212297370210293885719832,
                8696772024132538335301384667037109350601038575788952566222203798816610283060,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                15960707227058000517238706085810247102557195294914374463167646274777846249835,
                12600434264236862446033850254343057583464824912413717048793795328328330212605,
                15855756483319171890507298152964477634530258248748265716282797986058145266534,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ],
            [
                15846208915760731954667091811309260406550141924715915104219460769651268338431,
                14941305165750888611927396742686477481151661736272208988766205422971408648356,
                12862262498472802906442187303462198084729150273332424737168355509216970984467,
                20633846227573655562891472654875498275532732787736199734105126629336915134506
            ]
        ];
        var paths2rootToPos = [
            [
                0,
                0,
                1,
                0
            ],
            [
                0,
                0,
                0,
                0
            ],
            [
                1,
                0,
                1,
                0
            ],
            [
                0,
                0,
                0,
                0
            ]
        ];
        var fromX = [
            5188413625993601883297433934250988745151922355819390722918528461123462745458,
            1762022020655193103898710344498807340207430243997238950919845130297394445492,
            3765814648989847167846111359329408115955684633093453771314081145644228376874,
            5686635804472582232015924858874568287077998278299757444567424097636989354076
        ];
        var fromY = [
            12688531930957923993246507021135702202363596171614725698211865710242486568828,
            8832411107013507530516405716520512990480512909708424307433374075372921372064,
            9087768748788939667604509764703123117669679266272947578075429450296386463456,
            20652491795398389193695348132128927424105970377868038232787590371122242422611
        ];
        var fromIndex = [
            2,
            4,
            3,
            1
        ];
        var toX = [
            1762022020655193103898710344498807340207430243997238950919845130297394445492,
            0,
            14513915892014871125822366308671332087536577613591524212116219742227565204007,
            0
        ];
        var toY = [
            8832411107013507530516405716520512990480512909708424307433374075372921372064,
            0,
            6808129454002661585298671177612815470269050142983438156881769576685169493119,
            0
        ];
        var nonceFrom = [
            0,
            0,
            0,
            0
        ];
        var amount = [
            500,
            200,
            10,
            0
        ];
        var tokenTypeFrom =  [
            2,
            2,
            1,
            0
        ];
        var R8x = [
            881435600980839756557864672687352958246823615623067471382369537298639480473,
            2300755075168443291255333131763961974580906533695207131348693968861587339263,
            3195590963145840918059089506174009337457769759642475309956144584179783231111,
            8465255423668559610438823373646642266005408868646990631250385626976438883563
        ];
        var R8y = [
            7281408697071773036178843769513744105763140263113779889801337217142790667169,
            21537286765430414778692427500563354541479535018630291281567008614744314348792,
            12984862425413159340440376466106270573439564520656597515666971026916381354694,
            2723308881186539084515338610523052339122306443507329334121516397867219597117
        ];
        var S = [
            1741554106438246991550809820738277095116391101736136884539766215316428039644,
            634178895924607811185895857767020025754118655269974317285518979529308629543,
            776343858405278299091004755214235056330979994794904075404272701620470885730,
            2656710442651054479921937322651245544937826721419138119609359727378949657401
        ];
        var balanceFrom = [
            1000,
            700,
            20,
            0
        ];
        var balanceTo = [
            200,
            0,
            100,
            0
        ];
        var nonceTo = [
            0,
            0,
            0,
            0
        ];
        var tokenTypeTo = [
            2,
            0,
            1,
            0
        ];

        var n = 4;
        var m = 2;

        main.txRoot <== txRoot;
        main.currentState <== currentState;
        for (var i=0;i<2**m;i+=1) {
            for (var j=0;j<m;j+=1) {
                main.paths2txRoot[i][j] <== paths2txRoot[i][j];
                main.paths2txRootPos[i][j] <== paths2txRootPos[i][j];
            }

            for (var j=0;j<n;j+=1) {
                main.paths2rootFrom[i][j] <== paths2rootFrom[i][j];
                main.paths2rootTo[i][j] <== paths2rootTo[i][j];
                main.paths2rootFromPos[i][j] <== paths2rootFromPos[i][j];
                main.paths2rootToPos[i][j] <== paths2rootToPos[i][j];
            }

            main.balanceFrom[i] <== balanceFrom[i];
            main.fromIndex[i] <== fromIndex[i];            
            main.tokenTypeFrom[i] <== tokenTypeFrom[i];
            main.nonceFrom[i] <== nonceFrom[i];
            main.fromX[i] <== fromX[i];
            main.fromY[i] <== fromY[i];
            main.R8x[i] <== R8x[i];
            main.R8y[i] <== R8y[i];
            main.S[i] <== S[i];

            main.amount[i] <== amount[i];

            main.toX[i] <== toX[i];
            main.toY[i] <== toY[i];
            main.nonceTo[i] <== nonceTo[i];
            main.balanceTo[i] <== balanceTo[i];
            main.tokenTypeTo[i] <== tokenTypeTo[i];
        }

        for (var i=0;i<2**(m+1)+1;i+=1) {
            main.intermediateRoots[i] <== intermediateRoots[i];
        }

        main.out === newState;

    }
}


component main = Main(4,2);
