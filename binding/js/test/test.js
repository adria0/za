const circom2js = require("../lib/index.js");
const fs = require("fs");
const assert = require("chai").assert;

describe("Basic test", function () {

    this.timeout(5000);

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
      const verifying_key = circom2js.setupSync(circuit_path,pk_path,"json");
      
      all_inputs = { p:2, q:3 }
      proof_and_public_inputs = circom2js.proveSync(pk_path,JSON.stringify(all_inputs));
      
      const success = circom2js.verifySync(verifying_key,proof_and_public_inputs);
      assert.equal(success,true);

    });
});


