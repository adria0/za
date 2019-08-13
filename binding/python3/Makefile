.PHONY: test

SO_LIB=native/target/release/libcircom2py.so

all: $(SO_LIB)

$(SO_LIB):
	( cd native && cargo build --release)

test: $(SO_LIB)
	cp $(SO_LIB) test
	(cd test && python3 test.py)
