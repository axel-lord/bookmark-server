use actix_web::{web, App, HttpServer};

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("{cli:#?}");

    println!(env!("OUT_DIR"));

    let state = web::Data::new(cli);
    // println!("{}", get_file_content!("main.rs", cli));
    HttpServer::new(move || App::new().app_data(state.clone()))
        .workers(1)
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
}
