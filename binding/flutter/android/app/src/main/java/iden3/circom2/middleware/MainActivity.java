package iden3.circom2.middleware;

import android.os.Bundle;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.util.ArrayList;

import io.flutter.app.FlutterActivity;
import io.flutter.plugin.common.MethodChannel;
import io.flutter.plugin.common.MethodChannel.MethodCallHandler;
import io.flutter.plugin.common.MethodChannel.Result;
import io.flutter.plugin.common.MethodCall;
import io.flutter.plugins.GeneratedPluginRegistrant;

public class MainActivity extends FlutterActivity {

  static {
    System.loadLibrary("middleware");
  }

  private static final String MIDDLEWARE_CHANNEL = "iden3.circom2/middleware";

  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    GeneratedPluginRegistrant.registerWith(this);

    new MethodChannel(getFlutterView(), MIDDLEWARE_CHANNEL).setMethodCallHandler(
            new MethodCallHandler() {
              @Override
              public void onMethodCall(MethodCall call, Result result) {
                if (call.method.equals("middleWare")) {
                  Gson gsonBuilder = new GsonBuilder().create();
                  String args = gsonBuilder.toJson(call.arguments);

                  String res = middleWare(args);

                  if (res.length() > 0) {
                    result.success(res);
                  } else {
                    result.error("UNAVAILABLE", "Method unavailable.", null);
                  }
                } else {
                  result.notImplemented();
                }
              }
            }
    );
  }

  private String middleWare(String args) {
    MiddleWare m = new MiddleWare();
    return m.call(args);
  }
}
