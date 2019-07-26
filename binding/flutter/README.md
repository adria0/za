# Flutter circom2 plugin

Based on https://github.com/mimirblockchainsolutions/flutter-rust-middleware 

## Prerequisites

- [Rust](https://www.rust-lang.org)
- [Flutter](https://github.com/flutter/flutter)
- [cargo-lipo](https://github.com/TimNN/cargo-lipo)
- [Android Studio](https://developer.android.com/studio/)
- [NDK](https://developer.android.com/ndk/)
- [Xcode](https://developer.apple.com/xcode/)

## Getting Started

The first thing you need to do is set a few environment variables to help the process.

**note:** ***these need to be the location of the directories or your variables should reflect
the directories they are actually located at***

```
Mac:
  export ANDROID_HOME=/Users/$USER/Library/Android/sdk

  export NDK_HOME=$ANDROID_HOME/ndk-bundle

Linux:
  must add

```

Then, you need to run the ndk script to build your compile targets from the root folder of the project

`./ndk.sh`

Next, change to the cargo directory

`cd cargo`

and run the cargo build script

`./build.sh`

finally, you can use **flutter** to build the project for android or ios from the root directory of the project:

***(you must build with flutter before opening with android studio or Xcode)***

```
For iOS:
  flutter build ios

For Android:
  flutter build apk
```

