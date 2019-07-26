// Copyright 2017 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';

import 'dart:async';
import 'dart:io';


// command class lets us pass any method and parameter to the rust backend
class Command {
  Command(this.method, this.params);

  final String method;
  final dynamic params;
}


class MiddleWare {
  static const MethodChannel middlewareChannel = const MethodChannel('iden3.circom2/middleware');

  static Future<String> execute(Command cmd) async {
    String middleware;
    try {
      final String call = await middlewareChannel.invokeMethod(
          'middleWare',
          <String, dynamic>{
            'method': cmd.method,
            'params': cmd.params,
          }
      );

      // decode the output
      Map<String, dynamic> result = json.decode(call);
      // store the output string to return to the frontend
      middleware = '${result['Ok']}!';

    } on PlatformException {
      middleware = 'Failed to call ${cmd.method}.';
    }
    // return the result to the frontend
    return middleware;
  }
}

class PlatformChannel extends StatefulWidget {
  @override
  _PlatformChannelState createState() => new _PlatformChannelState();
}

class _PlatformChannelState extends State<PlatformChannel> {

  String _middleware = 'Call a method.';

  Future<String> get _localPath async {
    final directory = await getApplicationDocumentsDirectory();
    return directory.path;
  }

  Future<void> middleware() async {
    final path = await _localPath;
    
    final circuit= """
      template T() {
        signal private input p;
        signal private input q;
        signal output r;

        r <== p * q;

      }
      component main = T();    
    """;

    final provingKey = """
      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
      E7Ajkt/cMOj/OaMd0i8389BxwQ==    
    """.replaceAll("\n", "").replaceAll(" ", "");    

    final input = """{ "p" : "3", "q" : "7" }""";

    final circuitPath = '$path/circuit.circom';
    final provingKeyPath = '$path/proving.key';

    final circuitFile =  File(circuitPath);
    circuitFile.writeAsStringSync(circuit);

    final provingKeyFile =  File(provingKeyPath);
    provingKeyFile.writeAsBytesSync(base64Decode(provingKey));

    String middleware =  await MiddleWare.execute(
        new Command('prove',  [circuitPath, provingKeyPath, input] )
    );

    setState(() {
       _middleware = middleware;
    });
  }

  @override
  Widget build(BuildContext context) {
    return new Material(
      child: new Column(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: <Widget>[
          new Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              new Text(_middleware, key: const Key('Response')),
              new Padding(
                padding: const EdgeInsets.all(16.0),
                child: new RaisedButton(
                  child: const Text('Call'),
                  onPressed: () => middleware(),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

void main() {
 runApp(new MaterialApp(home: new PlatformChannel()));
}
