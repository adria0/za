package golang

/*
#cgo LDFLAGS: -L${SRCDIR}/native/target/release -lcircom2go

void verbose(int);
int setup(const char*, const char*,const char*,char*,size_t,char*,size_t);
int prove(const char*, const char*,const char*,char*,size_t,char*,size_t);
int verify(const char*, const char*,char*,size_t);
*/
import "C"

import (
    "errors"
    "encoding/json"
)

const (
	VerifierJSON = "json"
	VerifierSolidity = "solidity"
	errNone  = 0
	errBufferTooSmall = 1
	errVerificationFailed = 2
	errCustom = 100
)

var (
	ErrProofFailed = errors.New("proof failed")
	ErrBufferTooSmall = errors.New("buffer too small")
	ErrUnexpected = errors.New("unexpected result")
)

func Verbose(on bool) {
	if on {
		C.verbose(1)
	} else {
		C.verbose(0)
	}
}

func Setup(circuitPath string, pkPath string, verifierType string, maxBuffer uint) (string,error) {

    maxBufferC := (C.size_t)(maxBuffer)    
    retBufferC := (*C.char)(C.CBytes(make([]byte,maxBufferC)))
	errBufferC := (*C.char)(C.CBytes(make([]byte,maxBufferC)))

	circuitPathC := C.CString(circuitPath)
	pkPathC := C.CString(pkPath)
	verifierTypeC := C.CString(verifierType)

	result := C.setup(
		circuitPathC,
		pkPathC,
		verifierTypeC,
		retBufferC, maxBufferC,
		errBufferC, maxBufferC,
	)

	switch result {
	case errNone: return C.GoString(retBufferC),nil
	case errBufferTooSmall: return "", ErrBufferTooSmall
	case errCustom: return "", errors.New(C.GoString(errBufferC))
	}
	return "", ErrUnexpected
}

func Prove(circuitPath string, pkPath string, inputs interface{}, maxBuffer uint) (string,error) {
    maxBufferC := (C.size_t)(maxBuffer)    
	proofBufferC := (*C.char)(C.CBytes(make([]byte,maxBufferC)))
	errBufferC := (*C.char)(C.CBytes(make([]byte,maxBufferC)))

    circuitPathC := C.CString(circuitPath)
    pkPathC := C.CString(pkPath)    
    inputsJSON, err := json.Marshal(inputs)
    if err != nil {
        return "",err
    }

	inputsJSONC := C.CString(string(inputsJSON))

	result := C.prove(
		circuitPathC,
		pkPathC,
		inputsJSONC,
		proofBufferC, maxBufferC,
		errBufferC, maxBufferC,
	)

	switch result {
	case errNone: return C.GoString(proofBufferC),nil
	case errBufferTooSmall: return "", ErrBufferTooSmall
	case errCustom: return "", errors.New(C.GoString(errBufferC))
	}
	return "", ErrUnexpected
}

func Verify(verifyingKey string, proofWithInputs string, maxBuffer uint) (bool,error) {
    maxBufferC := (C.size_t)(maxBuffer)    
	errBufferC := (*C.char)(C.CBytes(make([]byte,maxBufferC)))

    verifyingKeyC := C.CString(verifyingKey)
    proofWithInputsC := C.CString(proofWithInputs)    

	result := C.verify(
		verifyingKeyC,
		proofWithInputsC,
		errBufferC, maxBufferC,
	)

	switch result {
    case errNone: return true,nil
    case errVerificationFailed: return false, nil
	case errBufferTooSmall: return false, ErrBufferTooSmall
	case errCustom: return false, errors.New(C.GoString(errBufferC))
	}
	return false, ErrUnexpected
}
