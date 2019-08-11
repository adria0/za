import libcircom2py as circom
import json

circuit_path = "/tmp/circuit.circom" 
pk_path = "/tmp/proving.key"
circuit = """
template T() {
        signal private input p;
        signal private input q;
        signal output r;

        r <== p*q;
}
component main = T();
"""

circom.verbose(True)

with open(circuit_path, 'w') as filehandle:
    filehandle.write(circuit)
      
verifying_key = circom.setup(circuit_path,pk_path,"json")

all_inputs = { "p":"2", "q":"3" }
proof_and_public_inputs = circom.prove(circuit_path,pk_path,json.dumps(all_inputs))
      
success = circom.verify(verifying_key,proof_and_public_inputs)
print("SUCCESS", success)