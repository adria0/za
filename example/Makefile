ZA=../target/release/za

all: proof.json

proof.json: $(ZA) proving.key input.json
	$(ZA) prove

proving.key: $(ZA) circuit.za
	$(ZA) setup

$(ZA):
	(cd .. && cargo build --release)

clean: 
	rm proof.json proving.key verifier.sol
