use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

mod server_dev {
    use clap::Parser;
    use std::path::PathBuf;
    use tokio::fs;

    #[derive(Parser, Debug)]
    pub struct Cli {
        server_root: PathBuf,
    }

    impl Cli {
        #[allow(dead_code)]
        pub async fn file_content(&self, path: &str) -> String {
            fs::read_to_string(
                [self.server_root.clone(), PathBuf::from(path)]
                    .iter()
                    .collect::<PathBuf>(),
            )
            .await
            .expect("file requested should exist")
        }
    }
}

mod not_server_dev {
    use clap::Parser;

    #[derive(Parser, Debug)]
    pub struct Cli {}
}

#[cfg(not(server_dev))]
use not_server_dev::*;
#[cfg(server_dev)]
use server_dev::*;

#[cfg(server_dev)]
#[allow(unused_macros)]
macro_rules! get_file_content {
    ($file_name:expr, $source_location:expr) => {
        $source_location.file_content($file_name).await
    };
}

#[cfg(not(server_dev))]
#[allow(unused_macros)]
macro_rules! get_file_content {
    ($file_name:expr, $source_location:expr) => {{
        let _ = &$source_location;
        include_str!($file_name)
    }};
}

#[get("/")]
async fn index(data: web::Data<Cli>) -> impl Responder {
    HttpResponse::Ok().body(get_file_content!("index.html", data))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("{cli:#?}");

    let state = web::Data::new(cli);
    // println!("{}", get_file_content!("main.rs", cli));
    HttpServer::new(move || App::new().app_data(state.clone()))
        .workers(1)
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
}
