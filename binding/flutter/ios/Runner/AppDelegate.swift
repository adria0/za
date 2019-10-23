// Copyright 2017 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import UIKit
import Flutter
import Foundation

@UIApplicationMain
@objc class AppDelegate: FlutterAppDelegate {

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplicationLaunchOptionsKey: Any]?) -> Bool {
    GeneratedPluginRegistrant.register(with: self);
    let controller : FlutterViewController = window?.rootViewController as! FlutterViewController;
    let middlewareChannel = FlutterMethodChannel.init(name: "iden3.za/middleware",
                                                   binaryMessenger: controller);
    middlewareChannel.setMethodCallHandler({
      (call: FlutterMethodCall, result: FlutterResult) -> Void in
      if ("middleWare" == call.method) {

        let jsonData = try! JSONSerialization.data(withJSONObject: call.arguments!, options: .prettyPrinted);
        let jsonString = NSString(data: jsonData, encoding: String.Encoding.utf8.rawValue)! as String;
 
        self.middleWare(args: jsonString, result: result);
      } else {
        result(FlutterMethodNotImplemented);
      }
    });

    return super.application(application, didFinishLaunchingWithOptions: launchOptions);
  }

    private func middleWare(args: String, result: FlutterResult) {
    let middleWare = MiddleWare();
    let reslt = middleWare.call(to: args);
    result(reslt);
  }

}
