#![warn(clippy::pedantic)]

const DEV_VAR_KEY: &str = "BOOKMARK_SERVER_DEV";
const DIRECTORY_LISTING: &str = "./serve.txt";

use proc_macro2::{Ident, Span};
use quote::quote;
use std::{
    env::{self, VarError},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use tap::Pipe;

#[derive(Debug, Clone, Copy)]
struct Mapping<T, U, V> {
    function: T,
    web: U,
    file: V,
}

type HomogenousMapping<T> = Mapping<T, T, T>;

async fn embed_html(out_file: &Path, mappings: &[HomogenousMapping<&'static str>]) {
    let futures = mappings.iter().map(
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

fn get_html(out_file: &Path, mappings: &[HomogenousMapping<&'static str>]) {
    let mut writer =
        BufWriter::new(File::create(out_file).expect("file creation in build dir should succeed"));
    for Mapping {
        function,
        web,
        file,
    } in mappings
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
    println!("cargo:rerun-if-env-changed={DEV_VAR_KEY}");
    println!("cargo:rerun-if-changed={DIRECTORY_LISTING}");

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR enviroment variable should exist");
    let out_file = Path::new(&out_dir).join("locations.rs");
    let mappings = tokio::fs::read_to_string(DIRECTORY_LISTING)
        .await
        .unwrap_or_else(|e| panic!("reading of {DIRECTORY_LISTING} should succeed, {e}"))
        .pipe(Box::<str>::from)
        .pipe(Box::leak)
        .lines()
        .map(|l| {
            let mut column_iter = l.split(',').map(str::trim).filter(|l| !l.is_empty());
            Mapping {
                function: column_iter
                    .next()
                    .expect("every row should have >=1 columns"),
                web: column_iter
                    .next()
                    .expect("every row should have >=2 columns"),
                file: column_iter
                    .next()
                    .expect("every row should have >=3 columns"),
            }
        })
        .collect::<Vec<_>>();

    match env::var(DEV_VAR_KEY) {
        Ok(val) if val.to_lowercase() == "true" => {
            println!("cargo:rustc-cfg=server_dev");
            get_html(&out_file, &mappings);
        }
        Err(VarError::NotPresent) | Ok(_) => {
            embed_html(&out_file, &mappings).await;
        }
        Err(e) => panic!("issue parsing enviroment variable ${{{DEV_VAR_KEY}}}, {e}"),
    };
}
