# ZA!
[![Build Status](https://github.com/adria0/za/workflows/Rust/badge.svg)](https://github.com/adria0/za/actions?query=workflow%3ARust)

An experimental port of IDEN3 circom compiler to rust with embeeded bellman-bn128 prover.

### Building 

Install rust

`curl https://sh.rustup.rs -sSf | sh`

Install additional dependencies, you may need to install the `clang` `build-essentials` and `openssl-dev`

Clone the repo

`git clone https://github.com/iden3/za.git`

Build

`cargo build --release`

The final binary will be in `target/release/za`

### Usage

#### Generating trusted setup

`za setup --circuit <circut.circom> --pk <proving.key> --verifier <verifier.sol>`

- `circuit.circom` is an input file with the `main` component that specifies the circuit
- `proving.key` if a generated output with the key required to generate proofs
- `verifier.sol` if a generated output with the smartcontract to verify the generated proofs

_if you want to do a test, create a file with name `circuit.circom` with the following contents and run the `za setup`_

```
template T() {
    signal private input p;
    signal private input q;
    signal output r;

    r <== p*q;
}
component main = T();
```

#### Generating a proof

`za prove --circuit <circuit.circom> --input <input.json> --pk <proving.key> --proof <proof.json>`

- `circuit.circom` is an input file with the `main` component that specifies the circuit
- `input.json` is an input file with the required input signals to generate the full witness
- `proving.key` if an input file with the key required to generate proofs
- `proof.json`  the input required to the smartcontract to verify the proof

_if you want to do a test, create a file with name `input.circom` with the following contents and run the `za prove`_

```
{ a : 2, b: 3 }
```

_then deploy the `verifier.sol` smartcontract and exec the `verifyTx` method with the contents of the `proof.json`_


#### Testing a circuit

In order to test if a circuit is correct is possible to write an embedded test by using the `#[test]` tag before a template definition (see `interop/circomlib/babyjub.circom`), to execute the test, run:

- `za test --circuit <circuit.circom>`

this will run the tests found in the circuit and all the tests found in the included templates

### Javascript bindings

to compile the javascript bindings, go to the `binding/js` folder and run:

- `npm i`
- `npm run install`
- `npm test`

check the test located in `binding/js/test/test.js`

### Flutter bindings

The code is based on https://github.com/mimirblockchainsolutions/flutter-rust-middleware 

#### Prerequisites

- [Rust](https://www.rust-lang.org)
- [Flutter](https://github.com/flutter/flutter)
- [cargo-lipo](https://github.com/TimNN/cargo-lipo)
- [Android Studio](https://developer.android.com/studio/)
- [NDK](https://developer.android.com/ndk/)
- [Xcode](https://developer.apple.com/xcode/)

Export vars

- `export ANDROID_HOME=/Users/$USER/Library/Android/sdk`
- `export NDK_HOME=$ANDROID_HOME/ndk-bundle`

Then, you need to run the ndk script to build your compile targets from the root folder of the project

`./ndk.sh`


#### Build

- Go to `binding/flutter/cargo` and run `./build.sh`
- Go to `binding/flutter` and run
    - `flutter build ios` or
    - `flutter build apk`

#### Test

- Go to `binding/flutter` and run `flutter run`


### Differences between official circom version

There are few differences between this implementation and the official circom:

- Precedence of operators rust-like instead C-like:
  - `DECNUMBER`, `HEXNUMBER`, `"(" exp ")"`
  - Unary `-` `!`
  - `**`      
  - `*` `/` `\\` `%`
  - `+` `-`     
  - `<<` `>>`  
  - `&` 
  - `^` 
  - `|` 
  - `==` `!=` `<` `>` `<=` `>=`
  - `&&`
  - `||`
- Removed `++`, `--` and `:?`
- Matrix access is only accessible with `[x][y]` (not with `[x,y]`) 
- End statement semicolons are mandatory
- Loops/conditionals statements must be inside blocks `{ }`
- Added `dbg!` function to trace variables, signals and components
- Do now allow to use component `signal output`s until all `signal input` are set  
- Signal input/outputs arrays should be evaluable with template parameters
- Stamements tagged with `#[w]` are only evaluated in witness generation
- `#[test]` tagged templates are used to verify embeeded tests
