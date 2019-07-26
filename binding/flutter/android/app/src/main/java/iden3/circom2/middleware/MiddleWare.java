package iden3.circom2.middleware;

public class MiddleWare {

    private static native String result(final String pattern);

    public String call(String to) {
        return result(to);
    }
}
