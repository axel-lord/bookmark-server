#![warn(clippy::pedantic)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};
use tap::Pipe;

const DIRECTORY_LISTING: &str = "./serve.txt";

fn leak_string(string: String) -> &'static str {
    Box::leak(string.into_boxed_str())
}

fn get_file_path(web_path: &str) -> &'static str {
    leak_string(format!(
        "./web{}{web_path}",
        if web_path.starts_with('/') { "" } else { "/" }
    ))
}

fn write_content(
    mut out: impl Write,
    mappings: impl Iterator<Item = [&'static str; 3]>,
    content: impl Fn([&'static str; 3]) -> io::Result<TokenStream>,
) -> io::Result<()> {
    for mapping in mappings {
        writeln!(out, "{}", content(mapping)?)?;
    }
    out.flush()
}

fn get_content([function, web, file]: [&'static str; 3]) -> io::Result<TokenStream> {
    let function = Ident::new(function, Span::call_site());
    quote! {
        #[get(#web)]
        async fn #function() -> impl Responder {
            HttpResponse::Ok().body(tokio::fs::read_to_string(#file).await.expect("file read should succeed"))
        }
    }.pipe(Ok)
}

fn embed_content([function, web, file]: [&'static str; 3]) -> io::Result<TokenStream> {
    println!("cargo:rerun-if-changed={file}");

    let function = Ident::new(function, Span::call_site());
    let content = fs::read_to_string(file)?;
    quote! {
        #[get(#web)]
        async fn #function() -> impl Responder {
            HttpResponse::Ok().body(#content)
        }
    }
    .pipe(Ok)
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed={DIRECTORY_LISTING}");
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR enviroment variable should exist");
    let out_file = Path::new(&out_dir).join("locations.rs");

    let mappings = fs::read_to_string(DIRECTORY_LISTING)?
        .pipe(leak_string)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut col_iter = l.splitn(2, ',').map(str::trim).inspect(|l| {
                assert!(
                    !l.is_empty(),
                    "one of the rows in {DIRECTORY_LISTING} contains an empty column"
                );
            });

            let function = col_iter.next().unwrap_or_else(|| {
                panic!("every row in {DIRECTORY_LISTING} should contain 2 or no columns, 0 found")
            });

            let web = col_iter.next().unwrap_or_else(|| {
                panic!("every row in {DIRECTORY_LISTING} should contain 2 or no columns, 1 found")
            });

            let file = get_file_path(web);

            [function, web, file]
        });

    let out_file = out_file.pipe(File::create)?.pipe(BufWriter::new);
    if cfg!(feature = "live") {
        println!("cargo:warning=compiling in \"live\" configuration, files will be read from disk on request, take care");
        write_content(out_file, mappings, get_content)
    } else {
        write_content(out_file, mappings, embed_content)
    }
}
