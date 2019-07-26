#!/bin/bash

rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release

if [[ $(rustc --print target-list) = *"apple-ios"* ]]; then
    cargo lipo --release
    ln -snf $PWD/target/universal/release/libmiddleware.a ../ios/Flutter/libmiddleware.a
fi

JNILIBS_DIR=$(cd ../android/app/src/main; pwd)/jniLibs
rm -rf $JNILIBS_DIR
mkdir $JNILIBS_DIR
mkdir $JNILIBS_DIR/arm64
mkdir $JNILIBS_DIR/armeabi
mkdir $JNILIBS_DIR/x86


ln -snf $PWD/target/aarch64-linux-android/release/libmiddleware.so ${JNILIBS_DIR}/arm64/libmiddleware.so
ln -snf $PWD/target/armv7-linux-androideabi/release/libmiddleware.so ${JNILIBS_DIR}/armeabi/libmiddleware.so
ln -snf $PWD/target/i686-linux-android/release/libmiddleware.so ${JNILIBS_DIR}/x86/libmiddleware.so

