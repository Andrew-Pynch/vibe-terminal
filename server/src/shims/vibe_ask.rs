use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, thread, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    agent_id: String,

    #[arg(long)]
    session_id: String,

    #[arg(long)]
    question: String,
}

#[derive(Serialize)]
struct AskPayload {
    agent_id: String,
    session_id: String,
    question: String,
}

#[derive(Deserialize, Debug)]
struct AskResponse {
    interaction_id: String,
}

#[derive(Deserialize, Debug)]
struct InteractionStatus {
    status: String, // "Pending" or "Resolved"
    result: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let server_url = env::var("VIBE_SERVER_URL").unwrap_or_else(|_| "http://localhost:4110".to_string());
    let client = Client::new();

    // 1. Register the question
    let response = client
        .post(format!("{}/agent/ask", server_url))
        .json(&AskPayload {
            agent_id: args.agent_id.clone(),
            session_id: args.session_id.clone(),
            question: args.question.clone(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        eprintln!("Failed to ask question: {}", response.status());
        std::process::exit(1);
    }

    let ask_res: AskResponse = response.json().await?;
    let interaction_id = ask_res.interaction_id;

    // println!("Question asked. Waiting for answer...");

    // 2. Poll for answer
    loop {
        let status_res = client
            .get(format!("{}/agent/ask/{}", server_url, interaction_id))
            .send()
            .await;

        match status_res {
            Ok(res) => {
                if res.status().is_success() {
                    let interaction: InteractionStatus = res.json().await?;
                    if interaction.status == "Resolved" {
                        if let Some(answer) = interaction.result {
                            println!("{}", answer);
                            break;
                        }
                    }
                }
            }
            Err(_) => {}
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
