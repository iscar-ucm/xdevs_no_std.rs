fn main() {
    // Enforce mutual exclusivity of features.
    let has_std = std::env::var("CARGO_FEATURE_STD").is_ok();
    let has_embassy = std::env::var("CARGO_FEATURE_EMBASSY").is_ok();

    if has_std && has_embassy {
        panic!("features 'std' and 'embassy' are mutually exclusive. Choose only one.");
    }
}
