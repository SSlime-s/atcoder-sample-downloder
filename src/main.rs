mod parser;
mod handle;
pub use crate::handle::AtCoder;

enum SubCommand {
    Download,
    Login,
}
impl SubCommand {
    fn value(&self) -> String {
        match *self {
            SubCommand::Download => "download".to_string(),
            SubCommand::Login => "login".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new("atcoder-sample-downloader")
        .version("1.0")
        .author("Hitoshi Togasaki. <togasakitogatoga+github.com>")
        .about(
            "Download sample test cases of AtCoder problem

Example:
    //Download
    atcoder-sample-donwloader download https://atcoder.jp/contests/agc035/tasks/agc035_a
    //Login
    atcoder-sample-donwloader login",
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Download.value())
                .about("Download sample test cases in your local")
                .arg(
                    clap::Arg::with_name("url")
                        .help("A URL of AtCoder problem")
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("with log")
                        .help("output sample contens")
                        .short("l")
                        .long("log")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Login.value())
                .about("Login AtCoder and save session in your local")
                .arg(clap::Arg::with_name("url").help("A login URL of AtCoder")),
        )
        .get_matches();

    //run sub commands
    let mut atcoder = AtCoder::new();
    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Download.value()) {
        match atcoder.download(matched.value_of("url").unwrap(), matched.is_present("with log")).await {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Login.value()) {
        match atcoder
            .login(
                matched
                    .value_of("url")
                    .unwrap_or("https://atcoder.jp/login"),
            )
            .await
        {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
