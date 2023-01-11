use std::env;
use copy_to_output::copy_to_output;

fn main() {
    // Re-runs script if any files in res are changed
    println!("cargo:rerun-if-changed=res/*");
    copy_to_output("config/log4rs.yaml", &env::var("PROFILE").unwrap()).expect("Could not copy");
}