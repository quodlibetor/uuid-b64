use error_chain::error_chain;

error_chain! {
    errors {
        ParseError(t: String) {
            description("Unable to parse UUID")
            display("Invalid Base64 representation for UUID: '{}'", t)
        }
    }
}
