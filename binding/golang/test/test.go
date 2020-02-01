package main

import "C"

import (
	za "github.com/adria0/za/binding/golang"
	"io/ioutil"
	"fmt"
)

func assert(e error) {
    if e != nil {
        panic(e)
    }
}

func main() {

	maxBuffer := uint(4000)

	circom.Verbose(true)

	circuitPath := "/tmp/circuit.circom" 
	pkPath := "/tmp/proving.key" 
	circuit := `
	template T() {
			signal private input p;
			signal private input q;
			signal output r;
	
			r <== p*q;
	}
	component main = T();
	`

	assert(ioutil.WriteFile(circuitPath, []byte(circuit), 0644))
	verifyingKey, err := za.Setup(circuitPath,pkPath,circom.VerifierJSON,maxBuffer)
	assert(err)

	inputs := map[string]string{
		"p": "2",
		"q": "3",
	}

	proofWithPublicInputs,err := za.Prove(pkPath,inputs,maxBuffer)
	assert(err)

	ok,err := za.Verify(verifyingKey,proofWithPublicInputs,maxBuffer)
	assert(err)
	fmt.Println(ok)
}
