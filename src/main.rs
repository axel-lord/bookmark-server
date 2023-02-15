use clap::Parser;
use std::env;

#[cfg(server_dev)]
use std::path::PathBuf;

#[cfg(server_dev)]
#[derive(Parser, Debug)]
struct Cli {
    server_root: PathBuf,
}

#[cfg(not(server_dev))]
#[derive(Parser, Debug)]
struct Cli {}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    #[cfg(server_dev)]
    {
        println!("Hello, server dev!");
    }

    #[cfg(not(server_dev))]
    {
        println!("Hello, normal dev!");
    }

    println!(
        "<{}>",
        env::var("BOOKMARK_SERVER_DEV").unwrap_or("false".to_string())
    );
    println!("{cli:#?}");
    Ok(())
}
