use crate::McpClient;
use clap::ArgMatches;

pub async fn handle_subcommand(matches: &ArgMatches) -> anyhow::Result<()> {
    let url = matches.get_one::<String>("url").expect("required argument");
    let mut client = McpClient::connect(url).await?;
    match matches.subcommand() {
        Some(("tool", matches)) => handle_tool(&client, matches).await?,
        _ => {
            anyhow::bail!("Unknown command");
        }
    }
    client.close().await?;
    Ok(())
}

async fn handle_tool(mcp: &McpClient, matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("list", _)) => handle_tool_list(mcp).await,
        Some(("call", matches)) => handle_tool_call(mcp, matches).await,
        _ => {
            anyhow::bail!("Unknown command for `tool`");
        }
    }
}

async fn handle_tool_list(mcp: &McpClient) -> anyhow::Result<()> {
    let tools = mcp.list_all_tools().await?;

    if tools.is_empty() {
        println!("No tools available.");
    } else {
        println!("Available tools ({}):", tools.len());
        println!("{}", "=".repeat(50));

        for tool in tools {
            println!("\n📦 {}", tool.name);
            if let Some(desc) = &tool.description {
                println!("{}", desc);
            }
        }
    }
    Ok(())
}

async fn handle_tool_call(mcp: &McpClient, matches: &ArgMatches) -> anyhow::Result<()> {
    let tool_name = matches
        .get_one::<String>("tool-name")
        .expect("`tool-name` is required")
        .clone();

    let arguments = matches
        .get_one::<String>("arguments")
        .map(|x| serde_json::from_str(x).unwrap());

    let result = mcp.call_tool(tool_name.clone(), arguments).await?;

    if matches!(result.is_error, Some(true)) {
        println!("⚠️Tool '{tool_name}' error");
    } else {
        println!("📦Tool '{tool_name}' result:");
    }
    println!("{}", "=".repeat(50));

    if let Some(structured) = &result.structured_content {
        println!("Structured: {}", serde_json::to_string_pretty(structured)?);
    }

    if !result.content.is_empty() {
        println!("\nText content:");
        for (i, content) in result.content.iter().enumerate() {
            if let Some(text) = content.as_text() {
                println!("[{}]: {}", i + 1, text.text);
            }
        }
    }

    Ok(())
}
