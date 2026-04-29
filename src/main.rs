use clap::{Arg, ArgGroup, Command};
use tracing::error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cfg_select! {
        debug_assertions => {
            tracing_subscriber::fmt::init();
        }
        _ => {
            use tracing_subscriber::util::SubscriberInitExt;
            use tracing_subscriber::*;

            let builder = fmt::Subscriber::builder();
            let builder = builder.with_max_level(tracing::Level::WARN);
            let builder = builder.with_target(false);
            let subscriber = builder.finish();
            subscriber.try_init().unwrap();
        }
    }

    let matches = Command::new("mcp-client")
        .about("A simple MCP client that supports tools/list and tools/call via streamable HTTP")
        .arg_required_else_help(true)
        .subcommand(Command::new("version").about("Prints the version information"))
        .subcommand(
            Command::new("tool")
                .about("tool operations to an MCP server")
                .arg_required_else_help(true)
                .arg(Arg::new("url").long("url").help("URL to the MCP server"))
                .arg(
                    Arg::new("unix-socket")
                        .long("unix")
                        .help("The unix socket to connect to the MCP server"),
                )
                .arg(
                    Arg::new("stdio")
                        .long("stdio")
                        .value_name("exe")
                        .help("Runs a MCP server by stdio transport. \
                            `exe` should be a piece of shell script which can be executed by `sh -c`")
                )
                .group(
                    ArgGroup::new("transport")
                        .args(["url", "unix-socket", "stdio"])
                        .required(true),
                )
                .subcommand(Command::new("list").about("lists all tools"))
                .subcommand(
                    Command::new("call")
                        .about("call a specific tool")
                        .arg(
                            Arg::new("tool-name")
                                .required(true)
                                .help("Name of the tool to call"),
                        )
                        .arg(
                            Arg::new("arguments")
                                .long("arg")
                                .help("Arguments for the tool call in JSON format"),
                        ),
                ),
        )
        .get_matches();

    match mcp_client::handle_subcommand(&matches).await {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(err) => {
            error!("{err}");
            std::process::exit(50);
        }
    }
}
