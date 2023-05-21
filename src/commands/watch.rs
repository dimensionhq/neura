use colored::Colorize;
use miette::Result;
use openai::chat::ChatCompletionMessage;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    time::Duration,
};

use crate::config::Config;

#[derive(PartialEq, Debug, Clone)]
pub struct Error {
    pub sha: String,
    pub file: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct Change {
    pub file: String,
    pub line_number: usize,
    pub new_line: String,
    pub time_estimate_seconds: u64,
}

#[derive(Deserialize)]
pub struct Changes {
    pub changes: Vec<Change>,
}

pub fn spawn_check() -> Vec<Error> {
    let mut errors = Vec::new();

    let mut child = Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start cargo check");

    if let Some(ref mut stdout) = child.stdout {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line.expect("Failed to read line");

            // Parse the JSON output and process it
            let json: serde_json::Value = serde_json::from_str(&line).unwrap();

            // Check if the JSON output is a compiler message
            if json["reason"] == "compiler-message" {
                // Check if the level is an error
                if json["message"]["level"] != "error" {
                    continue;
                }

                let rendered_error = json["message"]["rendered"].as_str().unwrap();

                // Check if spans exist
                let spans = json["message"]["spans"].as_array().unwrap();

                if spans.len() == 0 {
                    continue;
                }

                let file_name = json["message"]["spans"][0]["file_name"]
                    .as_str()
                    .unwrap()
                    .to_string();

                errors.push(Error {
                    sha: sha256::digest(format!("{}:{}", file_name, rendered_error)),
                    file: file_name,
                    message: rendered_error.to_string(),
                });
            }
        }
    }

    child.wait().expect("Failed to wait on child");

    errors
}

pub async fn execute() -> Result<()> {
    println!("⭐ Neura has joined your session.");

    let config = Config::load();

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message("💻 Running `cargo check` ...");
    spinner.enable_steady_tick(Duration::from_millis(100));
    let initial_errors = spawn_check();
    spinner.finish_and_clear();

    for error in initial_errors.iter() {
        // Read the contents of error.file
        let contents = std::fs::read_to_string(&error.file).unwrap();

        openai::set_key(std::env::var("NEURA_API_KEY").unwrap());

        // Start spinner
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_message("🐛 Debugging your issue ...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let prompt = format!(
            "You are an AI debugging copilot: fix this Rust error:\nCargo Error: {}\nFile Contents: {}\n\nRespond with a JSON. Use 'changes' for changes needed. Each change should have 'file' (filename), 'line_number' (line to be changed), 'new_line' (new line content), and 'time_estimate_seconds' (time to resolve manually). E.g.: {{\"changes\": [{{\"file\": \"src/main.rs\", \"line_number\": 3, \"new_line\": \"pub fn main() {{}}\", \"time_estimate_seconds\": 20}}]}}",
            error.message, contents
        );

        let bpe = tiktoken_rs::cl100k_base().unwrap();
        let prompt_tokens = bpe.encode_with_special_tokens(&prompt);

        // Prompt token count
        let prompt_token_count = prompt_tokens.len();
        println!("📝 Prompt token count: {}", prompt_token_count);

        let model = config.model.unwrap();

        let completion = openai::chat::ChatCompletion::builder(
            &model.code(),
            vec![ChatCompletionMessage {
                role: openai::chat::ChatCompletionMessageRole::User,
                content: prompt,
                name: None,
            }],
        )
        .max_tokens(300 as u64)
        .temperature(0.2)
        .create()
        .await
        .unwrap()
        .unwrap();

        // Stop spinner
        spinner.finish_and_clear();

        let returned_message = completion.choices.first().unwrap().message.clone();

        // println!(
        //     "🤖 The AI has responded with the following message: {}",
        //     returned_message.content
        // );

        let response_tokens = bpe.encode_with_special_tokens(&returned_message.content);

        // Prompt token count
        let response_token_count = response_tokens.len();

        // Convert the output into a json object
        let changes: Changes = serde_json::from_str(&returned_message.content).unwrap();

        let mut estimated_time = 0;

        for change in changes.changes {
            // Read the file again
            let mut contents = std::fs::read_to_string(&change.file).unwrap();

            // Apply the change
            let line_number = change.line_number;

            // Split the contents into lines
            let mut lines = contents.lines().collect::<Vec<&str>>();

            // Check if the line number is valid
            if line_number <= lines.len() + 1 {
                // Replace the line if it exists or add a new line
                println!(
                    "{} Editing {}, line {}",
                    ">".bright_black(),
                    change.file.bright_yellow(),
                    line_number
                );
                if line_number <= lines.len() {
                    lines[line_number - 1] = change.new_line.as_str();
                } else {
                    lines.push(change.new_line.as_str());
                }

                // Join the lines back together
                contents = lines.join("\n");

                // Write the new contents to the file
                std::fs::write(&change.file, &contents).unwrap();

                estimated_time += change.time_estimate_seconds;
            } else {
                println!("Error: Invalid line number {}", line_number);
            }
        }

        // Verify that the fixes applied have resolved the error
        let errors = spawn_check();

        let mut fixed_errors = 0;

        for old_error in &initial_errors {
            let still_exists = errors
                .iter()
                .any(|current_error| current_error.message == old_error.message);

            if !still_exists {
                fixed_errors += 1;
            }
        }

        // Assume a 50$/hr rate
        let rate_per_second: f64 = 50.0 / 3600.0;
        let cost_savings = estimated_time as f64 * rate_per_second;

        let mut prompt_cost: f64 = 0.0;
        let mut response_cost: f64 = 0.0;

        // Calcuate how much it cost to run the AI
        match model {
            crate::models::model::Model::GPT4 => {
                prompt_cost = (prompt_token_count as f64 / 1000.0) * 0.06;
                response_cost = (response_token_count as f64 / 1000.0) * 0.12;
            }
            crate::models::model::Model::GPT3Turbo => {
                prompt_cost = (prompt_token_count as f64 / 1000.0) * 0.002;
                response_cost = (response_token_count as f64 / 1000.0) * 0.002;
            }
            crate::models::model::Model::ClaudeV1 => {}
        }

        let total_cost = prompt_cost + response_cost;

        println!(
            "✅ Successfully resolved {} {}, saving you {}. {} remain.",
            fixed_errors.to_string().bright_green(),
            if fixed_errors == 1 { "error" } else { "errors" },
            format!("{:.2}$", (cost_savings - total_cost)).bright_cyan(),
            if errors.len() >= 1 {
                errors.len().to_string().bright_red()
            } else {
                "0".bright_green()
            }
        );
    }

    Ok(())
}
