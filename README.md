<h1 align="center">Za!</h1>

<p align="center">
    <a href="https://github.com/adria0/za/actions?query=workflow%3ARust"><img src="https://github.com/adria0/za/workflows/Rust/badge.svg"></a>
    <a href="https://codecov.io/gh/adria0/za"><img src="https://github.com/adria0/za/workflows/Code%20Coverage/badge.svg"></a>
    <a href="https://github.com/adria0/za/actions?query=workflow%3AClippy"><img src="https://github.com/adria0/za/workflows/Clippy/badge.svg"></a>
    <a href="https://github.com/adria0/za/actions?query=workflow%3ARustfmt"><img src="https://github.com/adria0/za/workflows/Rustfmt/badge.svg"></a>
    <a href="https://github.com/adria0/za/actions?query=workflow%3AAudit"><img src="https://github.com/adria0/za/workflows/Audit/badge.svg"></a>
    <img src="https://img.shields.io/badge/License-LGPLv2.1-blue.svg">
</p>

An experimental port of the [circom] zk-SNARK compiler in Rust with embedded bellman-bn128 prover. I created it as a PoC port of the existing JavaScript compiler to Rust when I was working for iden3. Since it was discontinued I forked it from https://www.github.com/iden3/za just to learn-by-doing.

**WARNING**: This is a proof-of-concept prototype, and in particular has not received careful code review.

[circom]: https://github.com/iden3/circom

### Building 

Install rust

`curl https://sh.rustup.rs -sSf | sh`

Install additional dependencies, you may need to install the `clang` `build-essentials` and `openssl-dev`

Clone the repo

`git clone https://github.com/adria0/za.git`

Build

`cargo build --release`

The final binary will be in `target/release/za`

### Usage

#### Generating trusted setup

`za setup --circuit <circut.za> --pk <proving.key> --verifier <verifier.sol> --verifiertype <solidity|json>`

- `circuit.za` is an input file with the `main` component that specifies the circuit
- `proving.key` is a generated output with the key required to generate proofs
- `verifier.sol` is a generated output with the smartcontract to verify the generated proofs

_if you want to do a test, create a file with name `circuit.za` with the following contents and run the `za setup`_

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

`za prove --input <input.json> --pk <proving.key> --proof <proof.json>`

- `input.json` is an input file with the required input signals to generate the full witness
- `proving.key` is an input file with the key required to generate proofs
- `proof.json` is the input required by the smartcontract to verify the proof

_if you want to do a test, create a file with name `input.json` with the following contents and run the `za prove`_

```
{ a : 2, b: 3 }
```

_then deploy the `verifier.sol` smartcontract and exec the `verifyTx` method with the contents of the `proof.json`_


#### Testing a circuit

In order to test if a circuit is correct is possible to write an embedded test by using the `#[test]` tag before a template definition (see `interop/circomlib/babyjub.circom`), to execute the test, run:

- `za test --circuit <circuit.za>`

this will run the tests found in the circuit and all the tests found in the included templates

### JavaScript bindings

to compile the JavaScript bindings, go to the `binding/js` folder and run:

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


### Differences with circom

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
- `#[]` expressions can be comment-scapped by using `/*#[]#*/` to be compatible with circom circuits. 
