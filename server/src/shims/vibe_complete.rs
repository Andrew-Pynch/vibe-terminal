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

    #[arg(short, long)]
    result_summary: Option<String>,
}

#[derive(Serialize, Debug)]
struct CompletePayload {
    agent_id: String,
    session_id: String,
    result_summary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let server_url = env::var("VIBE_SERVER_URL").unwrap_or_else(|_| "http://localhost:4110".to_string());
    let client = reqwest::Client::new();

    let payload = CompletePayload {
        agent_id: args.agent_id,
        session_id: args.session_id,
        result_summary: args.result_summary,
    };

    let res = client.post(format!("{}/agent/complete", server_url))
        .json(&payload)
        .send()
        .await?;

    res.error_for_status_ref()?;

    println!("Completion signal sent successfully: {:?}", payload);
    Ok(())
}
