//! # Z-Brain: AI-Shell CLI for NEURO-OS
//!
//! Z-Brain is an interactive command-line interface that wraps the NEURO-OS API,
//! providing a conversational shell experience with AI capabilities.
//!
//! ## Features
//! - Interactive chat with NEURO-OS AI
//! - Command history and auto-completion
//! - Multi-line input support
//! - Conversation session management
//! - Colorized output
//!
//! ## Usage
//! ```bash
//! zbrain              # Start interactive shell
//! zbrain "question"   # Quick query mode
//! zbrain --help       # Show help
//! ```

mod api;
mod config;
mod shell;

use anyhow::Result;
use clap::Parser;

/// Z-Brain: AI-powered command-line shell for NEURO-OS
#[derive(Parser, Debug)]
#[command(name = "zbrain")]
#[command(author = "NEURO-OS Team")]
#[command(version = "0.1.0")]
#[command(about = "AI-Shell CLI wrapper for NEURO-OS", long_about = None)]
struct Args {
    /// Quick query mode - send a single message and exit
    #[arg(index = 1)]
    query: Option<String>,

    /// API endpoint URL
    #[arg(short, long, default_value = "http://localhost:3000")]
    endpoint: String,

    /// Conversation ID to continue (optional)
    #[arg(short, long)]
    conversation: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load or create config
    let mut config = config::Config::load()?;
    config.api_endpoint = args.endpoint;
    config.verbose = args.verbose;
    config.colored = !args.no_color;

    // Create API client
    let client = api::NeuroClient::new(&config.api_endpoint);

    // Check connection
    if config.verbose {
        println!("Connecting to NEURO-OS at {}...", config.api_endpoint);
    }

    match client.health_check().await {
        Ok(healthy) if healthy => {
            if config.verbose {
                println!("Connected successfully!");
            }
        }
        _ => {
            eprintln!(
                "Warning: Could not connect to NEURO-OS at {}",
                config.api_endpoint
            );
            eprintln!("Make sure the backend is running.");
        }
    }

    // Run in appropriate mode
    if let Some(query) = args.query {
        // Quick query mode
        run_quick_query(&client, &query, args.conversation, &config).await
    } else {
        // Interactive shell mode
        shell::run_interactive_shell(client, config, args.conversation).await
    }
}

/// Execute a single query and print the response
async fn run_quick_query(
    client: &api::NeuroClient,
    query: &str,
    conversation_id: Option<String>,
    config: &config::Config,
) -> Result<()> {
    use colored::Colorize;

    let conv_id = conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    if config.verbose {
        println!("{}", "Sending query...".dimmed());
    }

    match client.chat(&conv_id, query).await {
        Ok(response) => {
            if config.colored {
                println!("{}", response.green());
            } else {
                println!("{}", response);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
