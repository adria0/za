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

import 'snarkcircuit.dart';
import 'snarkprovingkey.dart';

// command class lets us pass any method and parameter to the rust backend
class Command {
  Command(this.method, this.params);

  final String method;
  final dynamic params;
}


class MiddleWare {
  static const MethodChannel middlewareChannel = const MethodChannel('iden3.za/middleware');

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

      print("---result is "+call+"---");
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

  String _middleware = 'Snark it!';

  Future<String> get _localPath async {
    final directory = await getApplicationDocumentsDirectory();
    return directory.path;
  }

  Future<void> middleware() async {
    final path = await _localPath;
    
    final input = """
    {
    "privateKey" : "3876493977147089964395646989418653640709890493868463039177063670701706079087",
    "votingId" : "1",
    "nullifier" : "3642967737206797788953266792789642811467066441527044263456672813575084154491",
    "censusRoot" :"19335063285569462410966731541261274523671078966610109902395268519183816138000",
    "censusSiblings" : ["0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0"],
    "censusIdx" : "1337",
    "voteSigS":"1506558731080100151400643495683521762973142364485982380016443632063521613779",
    "voteSigR8x":"18137411472623093392316389329188709228585113201501107240811900197785235422996",
    "voteSigR8y":"3319823053651665777987773445125343092037295151949542813138094827788048737351",
    "voteValue":"2"
   }    
   """;

    final circuitPath = '$path/circuit.circom';
    final provingKeyPath = '$path/proving.key';

    final circuitFile =  File(circuitPath);
    circuitFile.writeAsStringSync(snarkCircuit);

    setState(() {
       _middleware ="Writing circuit and proving key...";
    });

    final provingKeyFile =  File(provingKeyPath);
    provingKeyFile.writeAsBytesSync(base64Decode(snarkProvingKey.replaceAll("\n", "").replaceAll(" ", "")));

    setState(() {
       _middleware ="Generating proof...";
    });


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
                  child: const Text('Vote'),
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
