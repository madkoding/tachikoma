//! Interactive shell implementation for Z-Brain CLI.
//!
//! Provides a REPL (Read-Eval-Print Loop) interface with:
//! - Command history and persistence
//! - Special commands (/, :, etc.)
//! - Multi-line input support
//! - Colorized output
//! - Session management

use crate::api::NeuroClient;
use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};

/// Special shell commands
const CMD_HELP: &str = "/help";
const CMD_QUIT: &str = "/quit";
const CMD_EXIT: &str = "/exit";
const CMD_CLEAR: &str = "/clear";
const CMD_NEW: &str = "/new";
const CMD_MODELS: &str = "/models";
const CMD_SEARCH: &str = "/search";

/// Run the interactive shell
pub async fn run_interactive_shell(
    client: NeuroClient,
    config: Config,
    initial_conversation: Option<String>,
) -> Result<()> {
    print_banner(&config);

    let mut editor = DefaultEditor::new()?;
    let mut conversation_id = initial_conversation.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Load history
    if let Ok(history_path) = config.history_path() {
        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }
    }

    // Print session info
    println!(
        "{}",
        format!("Session: {}", &conversation_id[..8]).dimmed()
    );
    println!("{}", "Type /help for commands, /quit to exit\n".dimmed());

    loop {
        let prompt = format!("{} ", "❯".cyan().bold());
        let readline = editor.readline(&prompt);

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                // Add to history
                let _ = editor.add_history_entry(input);

                // Handle special commands
                if input.starts_with('/') {
                    match handle_command(input, &client, &config, &mut conversation_id).await {
                        CommandResult::Continue => continue,
                        CommandResult::Exit => break,
                        CommandResult::Error(e) => {
                            eprintln!("{}: {}", "Error".red().bold(), e);
                            continue;
                        }
                    }
                }

                // Regular chat message
                print!("{}", "Thinking...".dimmed());
                std::io::Write::flush(&mut std::io::stdout())?;

                match client.chat(&conversation_id, input).await {
                    Ok(response) => {
                        // Clear "Thinking..." line
                        print!("\r{}\r", " ".repeat(20));

                        // Print response
                        println!("{}", format_response(&response, &config));
                        println!();
                    }
                    Err(e) => {
                        print!("\r{}\r", " ".repeat(20));
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "Use /quit to exit".dimmed());
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Ok(history_path) = config.history_path() {
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = editor.save_history(&history_path);
    }

    println!("{}", "Goodbye!".cyan());
    Ok(())
}

enum CommandResult {
    Continue,
    Exit,
    Error(String),
}

async fn handle_command(
    input: &str,
    client: &NeuroClient,
    config: &Config,
    conversation_id: &mut String,
) -> CommandResult {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd.as_str() {
        CMD_HELP => {
            print_help();
            CommandResult::Continue
        }
        CMD_QUIT | CMD_EXIT => CommandResult::Exit,
        CMD_CLEAR => {
            print!("\x1B[2J\x1B[1;1H");
            print_banner(config);
            CommandResult::Continue
        }
        CMD_NEW => {
            *conversation_id = uuid::Uuid::new_v4().to_string();
            println!(
                "{}",
                format!("New session: {}", &conversation_id[..8]).green()
            );
            CommandResult::Continue
        }
        CMD_MODELS => {
            match client.get_models().await {
                Ok(models) => {
                    println!("{}", "Available models:".cyan().bold());
                    for model in models {
                        println!("  • {}", model);
                    }
                }
                Err(e) => return CommandResult::Error(e.to_string()),
            }
            CommandResult::Continue
        }
        CMD_SEARCH => {
            if args.is_empty() {
                return CommandResult::Error("Usage: /search <query>".to_string());
            }
            match client.search_memories(args, 5).await {
                Ok(memories) => {
                    if memories.is_empty() {
                        println!("{}", "No memories found.".dimmed());
                    } else {
                        println!("{}", format!("Found {} memories:", memories.len()).cyan());
                        for mem in memories {
                            println!(
                                "  {} [{}] {}",
                                "•".green(),
                                mem.memory_type.yellow(),
                                mem.content
                            );
                        }
                    }
                }
                Err(e) => return CommandResult::Error(e.to_string()),
            }
            CommandResult::Continue
        }
        _ => CommandResult::Error(format!("Unknown command: {}. Type /help for commands.", cmd)),
    }
}

fn print_banner(config: &Config) {
    if config.colored {
        println!(
            r#"
{}
{}
{}
"#,
            "╔═══════════════════════════════════════╗".cyan(),
            "║       Z-Brain · NEURO-OS Shell        ║".cyan().bold(),
            "╚═══════════════════════════════════════╝".cyan()
        );
    } else {
        println!(
            r#"
╔═══════════════════════════════════════╗
║       Z-Brain · NEURO-OS Shell        ║
╚═══════════════════════════════════════╝
"#
        );
    }
}

fn print_help() {
    println!(
        r#"
{}

{}
  /help     Show this help message
  /quit     Exit the shell (or /exit)
  /clear    Clear the screen
  /new      Start a new conversation session
  /models   List available AI models
  /search   Search your memories (/search <query>)

{}
  • Type your message and press Enter to chat
  • Use Up/Down arrows to navigate history
  • Press Ctrl+C to cancel current input
  • Press Ctrl+D to exit
"#,
        "Z-Brain Commands".cyan().bold(),
        "Commands:".yellow(),
        "Tips:".yellow()
    );
}

fn format_response(response: &str, config: &Config) -> String {
    if config.colored {
        // Simple markdown-like formatting
        let mut result = String::new();
        for line in response.lines() {
            if line.starts_with("```") {
                result.push_str(&line.dimmed().to_string());
            } else if line.starts_with('#') {
                result.push_str(&line.yellow().bold().to_string());
            } else if line.starts_with("- ") || line.starts_with("* ") {
                result.push_str(&format!("  {} {}", "•".green(), &line[2..]));
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        result.trim_end().to_string()
    } else {
        response.to_string()
    }
}
