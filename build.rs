use std::env::{self, VarError};

fn main() {
    const DEV_VAR_KEY: &str = "BOOKMARK_SERVER_DEV";

    match env::var(DEV_VAR_KEY) {
        Ok(val) if val.to_lowercase() == "true" => {
            println!("cargo:rustc-cfg=server_dev")
        }
        Err(VarError::NotPresent) | Ok(_) => (),
        Err(e) => panic!("issue parsing enviroment variable ${{{DEV_VAR_KEY}}}, {e}"),
    };

    println!("cargo:rerun-if-env-changed={DEV_VAR_KEY}")
}
