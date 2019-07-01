include "../circomlib/mimc.circom";
include "../circomlib/eddsamimc.circom";
include "../circomlib/bitify.circom";
include "../circomlib/comparators.circom";
include "../circomlib/gates.circom";
include "./tx_existence_check.circom";
include "./balance_existence_check.circom";
include "./balance_leaf.circom";
include "./get_merkle_root.circom";

template Main(n,m) {
// n is depth of balance tree
// m is depth of transactions tree
// for each proof][ update 2**m transactions

    // Merkle root of transactions tree
    signal input tx_root;

    // Merkle proof for transaction in tx tree
    signal private input paths2tx_root[2**m][m];

    // binary vector indicating whether node in tx proof is left or right
    signal private input paths2tx_root_pos[2**m][m];

    // Merkle root of old balance tree
    signal input current_state;

    // intermediate roots (two for each tx)
    signal private input intermediate_roots[2**(m+1)];

    // Merkle proof for sender account in balance tree
    signal private input paths2root_from[2**m][n];

    // Merkle proof for receiver account in balance tree
    signal private input paths2root_to[2**m][n];

    // binary vector indicating whether node in balance proof for sender account
    // is left or right 
    signal private input paths2root_from_pos[2**m][n];

    // binary vector indicating whether node in balance proof for receiver account
    // is left or right 
    signal private input paths2root_to_pos[2**m][n];
    
    signal private input from_x[2**m]; //sender address x coordinate
    signal private input from_y[2**m]; //sender address y coordinate
    signal private input R8x[2**m]; // sender signature
    signal private input R8y[2**m]; // sender signature
    signal private input S[2**m]; // sender signature

    signal private input nonce_from[2**m]; // sender account nonce
    signal private input to_x[2**m]; // receiver address x coordinate
    signal private input to_y[2**m]; // receiver address y coordinate
    signal private input nonce_to[2**m]; // receiver account nonce
    signal private input amount[2**m]; // amount being transferred

    signal private input token_balance_from[2**m]; // sender token balance
    signal private input token_balance_to[2**m]; // receiver token balance
    signal private input token_type_from[2**m]; // sender token type
    signal private input token_type_to[2**m]; // receiver token type

    // new balance tree Merkle root
    signal output out;

    var NONCE_MAX_VALUE = 100;

    // constant zero address
                         
    var ZERO_ADDRESS_X = 0000000000000000000000000000000000000000000000000000000000000000000000000000;
    var ZERO_ADDRESS_Y = 00000000000000000000000000000000000000000000000000000000000000000000000000000;

    
    component txExistence[2**m - 1];
    component senderExistence[2**m - 1];
    component newSender[2**m - 1];
    component merkle_root_from_new_sender[2**m - 1];
    component receiverExistence[2**m - 1];
    component newReceiver[2**m - 1];
    component merkle_root_from_new_receiver[2**m - 1];

    component to_x_is_zero[2**m];
    component to_y_is_zero[2**m];
    signal to_x_is_not_zero[2**m];
    signal to_y_is_not_zero[2**m];

    component tx_y_not_zero_and_tx_y_not_zero[2**m];

    current_state === intermediate_roots[0];

    for (var i = 0; i < 2**m - 1; i+=1) {

        // transactions existence and signature check
        txExistence[i] = TxExistence(m);
        txExistence[i].from_x <== from_x[i];
        txExistence[i].from_y <== from_y[i];
        txExistence[i].to_x <== to_x[i];
        txExistence[i].to_y <== to_y[i];
        txExistence[i].nonce <== nonce_from[i];
        txExistence[i].amount <== amount[i];
        txExistence[i].token_type_from <== token_type_from[i];

        txExistence[i].tx_root <== tx_root;

        for (var j = 0; j < m; j+=1){
            txExistence[i].paths2_root_pos[j] <== paths2tx_root_pos[i][j] ;
            txExistence[i].paths2_root[j] <== paths2tx_root[i][j];
        }

        txExistence[i].R8x <== R8x[i];
        txExistence[i].R8y <== R8y[i];
        txExistence[i].S <== S[i];
    
        // sender existence check
        senderExistence[i] = BalanceExistence(n);
        senderExistence[i].x <== from_x[i];
        senderExistence[i].y <== from_y[i];
        senderExistence[i].token_balance <== token_balance_from[i];
        senderExistence[i].nonce <== nonce_from[i];
        senderExistence[i].token_type <== token_type_from[i];

        senderExistence[i].balance_root <== intermediate_roots[2*i];
        for (var j = 0; j < n; j+=1){
            senderExistence[i].paths2_root_pos[j] <== paths2root_from_pos[i][ j];
            senderExistence[i].paths2_root[j] <== paths2root_from[i][ j];
        }
    
        // balance checks
        //token_balance_from[i] - amount[i] <= token_balance_from[i];
        /*
        TODO
        
        token_balance_to[i] + amount[i] >= token_balance_to[i];

        nonce_from[i] != NONCE_MAX_VALUE;

        // check token types for non-withdraw transfers

        template IsZero() {
            signal input in;
            signal output out;
        }
        template IsEqual() {
            signal input in[2];
            signal output out;

        */
        
        to_x_is_zero[i] = IsEqual();
        to_x_is_zero[i].in[0] <== to_x[i];
        to_x_is_zero[i].in[1] <== ZERO_ADDRESS_X;
        to_x_is_not_zero[i] <== 1 - to_x_is_zero[i].out;

        to_y_is_zero[i] = IsEqual();
        to_y_is_zero[i].in[0] <== to_y[i];
        to_y_is_zero[i].in[1] <== ZERO_ADDRESS_Y;
        to_y_is_not_zero[i] <== 1 - to_y_is_zero[i].out;

        tx_y_not_zero_and_tx_y_not_zero[i] = ForceEqualIfEnabled();
        tx_y_not_zero_and_tx_y_not_zero[i].enabled <== to_x_is_not_zero[i] * to_y_is_not_zero[i];
        tx_y_not_zero_and_tx_y_not_zero[i].in[0] <== token_type_to[i];
        tx_y_not_zero_and_tx_y_not_zero[i].in[1] <== token_type_from[i];

/*

*/
        //if (to_x[i] != ZERO_ADDRESS_X && to_y[i] != ZERO_ADDRESS_Y){
        //    token_type_to[i] === token_type_from[i];
        //}

        // subtract amount from sender balance; increase sender nonce 
        newSender[i] = BalanceLeaf();
        newSender[i].x <== from_x[i];
        newSender[i].y <== from_y[i];
        newSender[i].token_balance <== token_balance_from[i] - amount[i];
        newSender[i].nonce <== nonce_from[i] + 1;
        newSender[i].token_type <== token_type_from[i];

        // get intermediate root from new sender leaf
        merkle_root_from_new_sender[i] = GetMerkleRoot(n);
        merkle_root_from_new_sender[i].leaf <== newSender[i].out;
        for (var j = 0; j < n; j+=1){
            merkle_root_from_new_sender[i].paths2_root[j] <== paths2root_from[i][ j];
            merkle_root_from_new_sender[i].paths2_root_pos[j] <== paths2root_from_pos[i][ j];
        }

        // check that intermediate root is consistent with input
        merkle_root_from_new_sender[i].out === intermediate_roots[2*i  + 1];

        // receiver existence check in intermediate root from new sender
        receiverExistence[i] = BalanceExistence(n);
        receiverExistence[i].x <== to_x[i];
        receiverExistence[i].y <== to_y[i];
        receiverExistence[i].token_balance <== token_balance_to[i];
        receiverExistence[i].nonce <== nonce_to[i];
        receiverExistence[i].token_type <== token_type_to[i];

        receiverExistence[i].balance_root <== intermediate_roots[2*i + 1];
        for (var j = 0; j < n; j+=1){
            receiverExistence[i].paths2_root_pos[j] <== paths2root_to_pos[i][ j] ;
            receiverExistence[i].paths2_root[j] <== paths2root_to[i][ j];
        }

        // add amount to receiver balance
        // if receiver is zero address][ do not change balance
        newReceiver[i] = BalanceLeaf();
        newReceiver[i].x <== to_x[i];
        newReceiver[i].y <== to_y[i];

        #[w]{ 
            if (to_x[i] == ZERO_ADDRESS_X && to_y[i] == ZERO_ADDRESS_Y){
                newReceiver[i].token_balance <== token_balance_to[i];
            }
            if (to_x[i] != ZERO_ADDRESS_X && to_y[i] != ZERO_ADDRESS_Y){
                newReceiver[i].token_balance <== token_balance_to[i] + amount[i];
            }
        }

        newReceiver[i].nonce <== nonce_to[i];
        newReceiver[i].token_type <== token_type_to[i];

        // get intermediate root from new receiver leaf
        merkle_root_from_new_receiver[i] = GetMerkleRoot(n);
        merkle_root_from_new_receiver[i].leaf <== newReceiver[i].out;
        for (var j = 0; j < n; j+=1){
            merkle_root_from_new_receiver[i].paths2_root[j] <== paths2root_to[i][ j];
            merkle_root_from_new_receiver[i].paths2_root_pos[j] <== paths2root_to_pos[i][ j];
        }

        // check that intermediate root is consistent with input
        merkle_root_from_new_receiver[i].out === intermediate_roots[2*i  + 2];

    }

    // final transactions existence and signature check
    component finalTxExistence = TxExistence(m);
    finalTxExistence.from_x <== from_x[2**m - 1];
    finalTxExistence.from_y <== from_y[2**m - 1];
    finalTxExistence.to_x <== to_x[2**m - 1];
    finalTxExistence.to_y <== to_y[2**m - 1];
    finalTxExistence.nonce <== nonce_from[2**m - 1];
    finalTxExistence.amount <== amount[2**m - 1];
    finalTxExistence.token_type_from <== token_type_from[2**m - 1];

    finalTxExistence.tx_root <== tx_root;

    for (var j = 0; j < m; j+=1){
        finalTxExistence.paths2_root_pos[j] <== paths2tx_root_pos[2**m - 1][ j] ;
        finalTxExistence.paths2_root[j] <== paths2tx_root[2**m - 1][ j];
    }

    finalTxExistence.R8x <== R8x[2**m - 1];
    finalTxExistence.R8y <== R8y[2**m - 1];
    finalTxExistence.S <== S[2**m - 1];

    // final sender existence check
    component final_sender_existence = BalanceExistence(n);
    final_sender_existence.x <== from_x[2**m - 1];
    final_sender_existence.y <== from_y[2**m - 1];
    final_sender_existence.token_balance <== token_balance_from[2**m - 1];
    final_sender_existence.nonce <== nonce_from[2**m - 1];
    final_sender_existence.token_type <== token_type_from[2**m - 1];
    
    final_sender_existence.balance_root <== intermediate_roots[2**(m + 1) - 2];

    for (var jj = 0; jj < n; jj+=1){
        final_sender_existence.paths2_root_pos[jj] <== paths2root_from_pos[2**m - 1][ jj] ;
        final_sender_existence.paths2_root[jj] <== paths2root_from[2**m - 1][ jj];
    }

    // final balance checks
    /*
    TODO
    token_balance_from[2**m - 1] - amount[2**m - 1] <= token_balance_from[2**m - 1];
    token_balance_to[2**m - 1] + amount[2**m - 1] >= token_balance_to[2**m - 1];

    nonce_from[2**m - 1] != NONCE_MAX_VALUE;
    
    // check token types for non-withdraw transfers
    */
    #[w] {
        if (to_x[2**m - 1] != ZERO_ADDRESS_X && to_y[2**m - 1] != ZERO_ADDRESS_Y){
            token_type_to[2**m - 1] === token_type_from[2**m - 1];
        }
    }

    // update final sender leaf
    component new_final_sender = BalanceLeaf();
    new_final_sender.x <== from_x[2**m - 1];
    new_final_sender.y <== from_y[2**m - 1];
    new_final_sender.token_balance <== token_balance_from[2**m - 1] - amount[2**m - 1];
    new_final_sender.nonce <== nonce_from[2**m - 1] + 1;
    new_final_sender.token_type <== token_type_from[2**m - 1];

    // get intermediate root from new final sender leaf
    component merkle_root_from_new_final_sender = GetMerkleRoot(n);
    merkle_root_from_new_final_sender.leaf <== new_final_sender.out;
    for (var j = 0; j < n; j+=1){
        merkle_root_from_new_final_sender.paths2_root[j] <== paths2root_from[2**m - 1][ j];
        merkle_root_from_new_final_sender.paths2_root_pos[j] <== paths2root_from_pos[2**m - 1][ j];
    }

    // check that intermediate root is consistent with input
    merkle_root_from_new_final_sender.out === intermediate_roots[2**(m + 1) - 1];

    // final receiver existence check using root from new final sender
    component final_receiver_existence = BalanceExistence(n);
    final_receiver_existence.x <== to_x[2**m - 1];
    final_receiver_existence.y <== to_y[2**m - 1];
    final_receiver_existence.token_balance <== token_balance_to[2**m - 1];
    final_receiver_existence.nonce <== nonce_to[2**m - 1];
    final_receiver_existence.token_type <== token_type_to[2**m - 1];
    
    final_receiver_existence.balance_root <== intermediate_roots[2**(m + 1) - 1];
    for (var j = 0; j < n; j+=1){
        final_receiver_existence.paths2_root_pos[j] <== paths2root_to_pos[2**m - 1][ j] ;
        final_receiver_existence.paths2_root[j] <== paths2root_to[2**m - 1][ j];
    }

    component new_final_receiver = BalanceLeaf();
    new_final_receiver.x <== to_x[2**m - 1];
    new_final_receiver.y <== to_y[2**m - 1];

    #[w] {
        if (to_x[2**m - 1] == ZERO_ADDRESS_X && to_y[2**m - 1] == ZERO_ADDRESS_Y){
            new_final_receiver.token_balance <== token_balance_to[2**m - 1];
        }
        if (to_x[2**m - 1] != ZERO_ADDRESS_X && to_y[2**m - 1] != ZERO_ADDRESS_Y){
            new_final_receiver.token_balance <== token_balance_to[2**m - 1] + amount[2**m - 1];
        }
    }

    new_final_receiver.nonce <== nonce_to[2**m - 1];
    new_final_receiver.token_type <== token_type_to[2**m - 1];

    component merkle_root_from_new_final_receiver = GetMerkleRoot(n);
    merkle_root_from_new_final_receiver.leaf <== new_final_receiver.out;
    for (var j = 0; j < n; j+=1){
        merkle_root_from_new_final_receiver.paths2_root[j] <== paths2root_to[2**m - 1][ j];
        merkle_root_from_new_final_receiver.paths2_root_pos[j] <== paths2root_to_pos[2**m - 1][ j];
    }

    // circuit outputs new balance root
    out <== merkle_root_from_new_final_receiver.out;

}

#[test] 
template test_rollup_42() {

    var out = 11906244236079007243914245890798662246316717426994617587947898072260792776007;
    var tx_root = 655926317945542797074993632917009959927083450268202759196407232656349645955;
    var current_state = 18648130918012499145379763689148123788934814534108898637620846058357562030290;

    var paths2tx_root = [
        [
            7920182099730273754772292249087557251748013756095841609029185132151132280465,
            17087000614363200202425883762664953902375829596010825089495865991695670487372
        ],
        [
            9147603832976282322177802237335033023892550539422420889941340634444804843588,
            17087000614363200202425883762664953902375829596010825089495865991695670487372
        ],
        [
            7543596251292424187454580071286604312359391743498405157449832911397497216395,
            8409055543623039321311148416467367238227979869198972778447826812079972187336
        ],
        [
            13909176565871710875093016924602387024278552150611849669211607776828222469413,
            8409055543623039321311148416467367238227979869198972778447826812079972187336
        ]
    ];
    var paths2tx_root_pos = [
        [0,0],
        [1,0],
        [0,1],
        [1,1]
    ];
    var intermediate_roots = [
        18648130918012499145379763689148123788934814534108898637620846058357562030290,
        13984568938318779243384249857072528475334097093971526730313818037575303895156,
        7912212943187484548235253028205895169032161272943992019907208995859312919611,
        16950419554751951994045449366301225585280586440249526156204829834323753076899,
        16950419554751951994045449366301225585280586440249526156204829834323753076899,
        20041328540498103896130506725191726766126968064550434617504774768356266583517,
        16416606783193441312741953679831197136946893312976088673969971211837418012007,
        11906244236079007243914245890798662246316717426994617587947898072260792776007
    ];
    var paths2root_from = [
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
    var paths2root_to = [
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
    var paths2root_from_pos = [
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
    var paths2root_to_pos = [
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
    var from_x = [
        5188413625993601883297433934250988745151922355819390722918528461123462745458,
        1762022020655193103898710344498807340207430243997238950919845130297394445492,
        3765814648989847167846111359329408115955684633093453771314081145644228376874,
        5686635804472582232015924858874568287077998278299757444567424097636989354076
    ];
    var from_y = [
        12688531930957923993246507021135702202363596171614725698211865710242486568828,
        8832411107013507530516405716520512990480512909708424307433374075372921372064,
        9087768748788939667604509764703123117669679266272947578075429450296386463456,
        20652491795398389193695348132128927424105970377868038232787590371122242422611
    ];
    var R8x = [
        13089479332832359283331901498973359098604209287063651100290375278730465236798,
        7983291864861647204354763508579435487011989337395325432834226412873248438045,
        14468712286173915750166651680918317301160985822896377740814417043118469755506,
        1985053535665938243219811857757911648887787617300571058863392598564636685902
    ];
    var R8y = [
        2637408348218936793024041284273045758414702791006977157843179199977771399476,
        701934180936084689159265663707026907937317273536866905652745094952975869069,
        14992734494135403876190625145808022394495261351375990015452342997829158458671,
        10477538069209035020457941813173449519572391779174101110543616768942550155879
    ];
    var S = [
        463482202707284636646722392808626991498971229083195436613133096224442269653,
        1051608622726488744913343337593760563634607176664541296623875563476429355114,
        543287037210896922722560565556285483406950788358712408477160228918928758188,
        1735335888050405736877788052494039451026458370885773708635665264508355836102
    ];
    var nonce_from = [
        0,
        0,
        0,
        0
    ];
    var to_x = [
        1762022020655193103898710344498807340207430243997238950919845130297394445492,
        0,
        14513915892014871125822366308671332087536577613591524212116219742227565204007,
        0
    ];
    var to_y = [
        8832411107013507530516405716520512990480512909708424307433374075372921372064,
        0,
        6808129454002661585298671177612815470269050142983438156881769576685169493119,
        0
    ];
    var nonce_to = [
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
    var token_balance_from = [
        1000,
        700,
        20,
        0
    ];
    var token_balance_to = [
        200,
        0,
        100,
        0
    ];
    var token_type_from = [
        2,
        2,
        1,
        0
    ];
    var token_type_to = [
        2,
        0,
        1,
        0
    ];
    component main =Main(4,2);
    var n = 4;
    var m = 2;

    main.tx_root <== tx_root;
    main.current_state <== current_state;
    for (var i=0;i<2**m;i+=1) {
        for (var j=0;j<m;j+=1) {
            main.paths2tx_root[i][j] <== paths2tx_root[i][j];
            main.paths2tx_root_pos[i][j] <== paths2tx_root_pos[i][j];
        }

        for (var j=0;j<n;j+=1) {
            main.paths2root_from[i][j] <== paths2root_from[i][j];
            main.paths2root_to[i][j] <== paths2root_to[i][j];
            main.paths2root_from_pos[i][j] <== paths2root_from_pos[i][j];
            main.paths2root_to_pos[i][j] <== paths2root_to_pos[i][j];
        }

        main.from_x[i] <== from_x[i];
        main.from_y[i] <== from_y[i];
        main.R8x[i] <== R8x[i];
        main.R8y[i] <== R8y[i];
        main.S[i] <== S[i];

        main.nonce_from[i] <== nonce_from[i];
        main.to_x[i] <== to_x[i];
        main.to_y[i] <== to_y[i];
        main.nonce_to[i] <== nonce_to[i];
        main.amount[i] <== amount[i];

        main.token_balance_from[i] <== token_balance_from[i];
        main.token_balance_to[i] <== token_balance_to[i];
        main.token_type_from[i] <== token_type_from[i];
        main.token_type_to[i] <== token_type_to[i];
    }

    for (var i=0;i<2**(m+1);i+=1) {
        main.intermediate_roots[i] <== intermediate_roots[i];
    }

    main.out === out;

}

component main=Main(4,2);

