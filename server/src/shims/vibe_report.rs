use clap::Parser;
use serde::Serialize;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    agent_id: String,

    #[arg(short, long)]
    session_id: String,

    #[arg(short, long, default_value_t = 0)]
    progress: u8,

    #[arg(short, long)]
    thought: Option<String>,
}

#[derive(Serialize, Debug)]
struct ReportPayload {
    agent_id: String,
    session_id: String,
    progress: u8,
    thought: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let server_url = env::var("VIBE_SERVER_URL").unwrap_or_else(|_| "http://localhost:4110".to_string());
    let client = reqwest::Client::new();

    let payload = ReportPayload {
        agent_id: args.agent_id,
        session_id: args.session_id,
        progress: args.progress,
        thought: args.thought,
    };

    let res = client.post(format!("{}/agent/report", server_url))
        .json(&payload)
        .send()
        .await?;

    res.error_for_status_ref()?;

    println!("Report sent successfully: {:?}", payload);
    Ok(())
}
