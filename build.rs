use quote::quote;
use std::{
    env::{self, VarError},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const DEV_VAR_KEY: &str = "BOOKMARK_SERVER_DEV";
const MAPPINGS: [(&str, &str); 1] = [("index.html", "./web/index.html")];

async fn embed_html(out_file: &Path) {
    let futures =
        MAPPINGS.map(|(web_path, file_path)| (web_path, tokio::fs::read_to_string(file_path)));

    let mut writer =
        BufWriter::new(File::create(out_file).expect("file creation in build dir should succeed"));
    for (_web_path, future) in futures {
        let content = future.await.expect("file and content should exist");
        writeln!(
            writer,
            "{}",
            quote! {
                fn hello() -> &'static str {
                    #content;
                }
            }
        )
        .expect("write should succeed");
    }
    writer.flush().expect("flushing of writer should succeed");
}
async fn get_html(_out_file: &Path) {}

#[tokio::main]
async fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR enviroment variable should exist");
    let out_file = Path::new(&out_dir).join("locations.rs");

    match env::var(DEV_VAR_KEY) {
        Ok(val) if val.to_lowercase() == "true" => {
            println!("cargo:rustc-cfg=server_dev");
            get_html(&out_file).await;
        }
        Err(VarError::NotPresent) | Ok(_) => {
            embed_html(&out_file).await;
        }
        Err(e) => panic!("issue parsing enviroment variable ${{{DEV_VAR_KEY}}}, {e}"),
    };

    println!("cargo:rerun-if-env-changed={DEV_VAR_KEY}")
}
