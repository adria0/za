class MiddleWare {
    func call(to: String) -> String {
        let result = request_function(to)
        let swift_result = String(cString: result!)
        function_free(UnsafeMutablePointer(mutating: result))
        return swift_result
    }
}
