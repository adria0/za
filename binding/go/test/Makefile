LIB_PATH=../../c/native
LIB_SO_PATH= $(LIB_PATH)/target/release
LIB_SO=$(LIB_SO_PATH)/za2c.so

test: $(LIB_SO) 
	LD_LIBRARY_PATH=$(LIB_SO_PATH) go run test.go

$(LIB_SO):
	cd $(LIB_PATH) && cargo build --release 

