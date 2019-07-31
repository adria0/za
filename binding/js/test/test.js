const circom2js = require("../lib/index.js");
const fs = require("fs");

describe("Basic test", function () {

    const circuit_path = "/tmp/circuit.circom"; 
    const pk_path = "/tmp/proving.key"; 

    it("Test simple circuit", async () => {

       const circuit = `
        template T() {
              signal private input p;
              signal private input q;
              signal output r;
      
              r <== p*q;
        }
        component main = T();
      `;
      
      fs.writeFileSync(circuit_path,circuit);
      
      circom2js.verbose(true)
      console.log(circom2js.setupSync(circuit_path,pk_path,"json"));
      
      all_inputs = { p:2, q:3 }
      proof_and_public_inputs = circom2js.proveSync(circuit_path,pk_path,JSON.stringify(all_inputs))
      proof_and_public_inputs = JSON.parse(proof_and_public_inputs)
      
      console.log(proof_and_public_inputs);

    });
});


