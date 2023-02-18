use proc_macro2::{Ident, Span};
use quote::quote;
use std::{
    env::{self, VarError},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug, Clone, Copy)]
struct Mapping<T = &'static str, U = &'static str, V = &'static str> {
    function: T,
    web: U,
    file: V,
}

impl<T, U, V> Mapping<T, U, V> {
    const fn new(function: T, web: U, file: V) -> Self {
        Self {
            function,
            web,
            file,
        }
    }
}

const DEV_VAR_KEY: &str = "BOOKMARK_SERVER_DEV";
const MAPPINGS: [Mapping; 1] = [Mapping::new("index", "index.html", "./web/index.html")];

async fn embed_html(out_file: &Path) {
    let futures = MAPPINGS.map(
        |Mapping {
             function,
             web,
             file,
         }| Mapping {
            function: Ident::new(function, Span::call_site()),
            web,
            file: (file, tokio::fs::read_to_string(file)),
        },
    );

    let mut writer =
        BufWriter::new(File::create(out_file).expect("file creation in build dir should succeed"));
    for Mapping {
        function,
        web,
        file: (path, file),
    } in futures
    {
        let content = file.await.expect("file and content should exist");
        let content = quote! {
            #[get(#web)]
            async fn #function() -> impl Responder {
                HttpResponse::Ok().body(#content)
            }
        };
        println!("cargo:rerun-if-changed={path}");
        writeln!(writer, "{content}").expect("write should succeed");
    }
    writer.flush().expect("flushing of writer should succeed");
}
async fn get_html(out_file: &Path) {
    let mut writer =
        BufWriter::new(File::create(out_file).expect("file creation in build dir should succeed"));
    for Mapping {
        function,
        web,
        file,
    } in MAPPINGS
    {
        let function = Ident::new(function, Span::call_site());
        let content = quote! {
            #[get(#web)]
            async fn #function() -> impl Responder {
                HttpResponse::Ok().body(tokio::fs::read_to_string(#file).await.expect("file read should succeed"))
            }

        };
        writeln!(writer, "{content}").expect("write should succeed");
    }
    writer.flush().expect("flushing of writer should succeed");
}

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
