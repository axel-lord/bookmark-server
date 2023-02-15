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
macro_rules! get_file_content {
    ($file_name:expr, $source_location:expr) => {
        $source_location.file_content($file_name).await.as_str()
    };
}

#[cfg(not(server_dev))]
macro_rules! get_file_content {
    ($file_name:expr, $source_location:expr) => {
        include_str!($file_name)
    };
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("{cli:#?}");
    println!("{}", get_file_content!("main.rs", cli));
    Ok(())
}
