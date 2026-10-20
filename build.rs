fn main() {
    // The async runtime backends are mutually exclusive.
    let has_tokio = std::env::var("CARGO_FEATURE_TOKIO").is_ok();
    let has_embassy = std::env::var("CARGO_FEATURE_EMBASSY").is_ok();

    if has_tokio && has_embassy {
        panic!("features 'tokio' and 'embassy' are mutually exclusive. Choose only one.");
    }
}
