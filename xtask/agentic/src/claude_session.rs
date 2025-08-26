use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use xtask::project_root;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgenticState {
    pub current_node: String,
    pub completed_nodes: Vec<String>,
    pub errors: Vec<String>,
    pub in_progress_nodes: HashMap<String, u32>, // node -> iteration count
}

impl AgenticState {
    pub fn new() -> Self {
        Self {
            current_node: String::new(),
            completed_nodes: Vec::new(),
            errors: Vec::new(),
            in_progress_nodes: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let path = project_root().join("xtask/agentic/agentic_state.json");
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = project_root().join("xtask/agentic/agentic_state.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

pub struct ClaudeSession {
    conversation_id: Option<String>,
}

impl ClaudeSession {
    pub fn new() -> Self {
        Self {
            conversation_id: None,
        }
    }

    pub fn call_claude(&mut self, prompt: &str, new_conversation: bool) -> Result<String> {
        // Create a temporary file for the prompt
        let temp_file = "/tmp/claude_prompt.txt";
        fs::write(temp_file, prompt)?;

        let mut cmd = Command::new("claude");
        cmd.arg("--print") // Non-interactive mode
            .arg("--output-format")
            .arg("json")
            .env_remove("ANTHROPIC_API_KEY") // Unset API key to use CLI auth
            .current_dir(project_root()); // Set working directory to monorepo root

        // Reuse conversation if we have one and not explicitly starting new
        if !new_conversation && self.conversation_id.is_some() {
            if let Some(ref conv_id) = self.conversation_id {
                cmd.arg("--resume").arg(conv_id);
            }
        }

        // Read prompt from file and pass it as argument
        let prompt = fs::read_to_string(temp_file)?;
        cmd.arg(&prompt);

        println!(
            "Calling Claude with prompt: {}",
            &prompt.lines().next().unwrap_or("")
        );

        let output = cmd
            .output()
            .context("Failed to execute claude CLI. Make sure it's installed and in PATH")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Claude CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON response to extract session ID and result
        let json_response: serde_json::Value = serde_json::from_str(&stdout)
            .context("Failed to parse JSON response from Claude CLI")?;

        // Extract session ID for future requests
        if let Some(session_id) = json_response.get("session_id").and_then(|s| s.as_str()) {
            self.conversation_id = Some(session_id.to_string());
        }

        // Extract the actual result content
        let result = json_response
            .get("result")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        Ok(result)
    }
}
