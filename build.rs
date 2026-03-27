fn main() {
    // Enforce mutual exclusivity: only std OR embassy
    let has_std = cfg!(feature = "std");
    let has_embassy = cfg!(feature = "embassy");

    if has_std && has_embassy {
        panic!("features 'std' and 'embassy' are mutually exclusive. Choose only one.");
    }

    if !has_std && !has_embassy {
        panic!("exactly one of 'std' or 'embassy' feature must be enabled");
    }
}
