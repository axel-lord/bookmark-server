use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {}

include!(concat!(env!("OUT_DIR"), "/locations.rs"));

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let state = web::Data::new(cli);

    HttpServer::new(move || App::new().app_data(state.clone()).service(index))
        .workers(1)
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
}
