fn main() {
    // Collect backend features that are mutually exclusive (std vs embassy).
    // rayon-backend is NOT a backend — it's a parallelization layer that works on top of std-backend.
    let backends: Vec<_> = std::env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("CARGO_FEATURE_") && key.ends_with("_BACKEND") {
                Some(key[14..].to_ascii_lowercase()) // Strip 'CARGO_FEATURE_'
            } else {
                None
            }
        })
        .filter(|name| *name != "rayon_backend") // rayon-backend co-exists with std-backend
        .collect();

    if backends.len() > 1 {
        panic!("Multiple backend features selected: {:?}", backends);
    }
}
