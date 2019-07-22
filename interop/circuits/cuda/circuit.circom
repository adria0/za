template T(N) {
	signal private input p;
	signal output q;

	signal tmp[N];	
	
	tmp[0] <== p;
	tmp[1] <== p + 1;
	tmp[2] <== p + 2;
	
	for (var i=0;i<N-3;i+=1) {
		tmp[i+3] <== (tmp[i] + tmp[i+1]) * (tmp[i+1] + tmp[i+2]);
	}

	q <== tmp[N-1];
}

#[test]
template test1() {
	component main = T(10000);
	#[w] {
		main.p <== 2;
	}
}

component main = T(10000);
