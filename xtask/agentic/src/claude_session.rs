use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;
use xtask::project_root;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgenticState {
    pub current_node: String,
    pub completed_nodes: Vec<String>,
    pub errors: Vec<String>,
    pub in_progress_nodes: HashMap<String, u32>, // node -> iteration count
    pub node_structure_cache: HashMap<String, String>, // node -> structure analysis
    pub node_match_updated: std::collections::HashSet<String>, // nodes already added to match statement
}

impl AgenticState {
    pub fn new() -> Self {
        Self {
            current_node: String::new(),
            completed_nodes: Vec::new(),
            errors: Vec::new(),
            in_progress_nodes: HashMap::new(),
            node_structure_cache: HashMap::new(),
            node_match_updated: HashSet::new(),
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

    pub fn clear_node_structure_cache(&mut self) -> Result<()> {
        eprintln!("Clearing node structure cache (e.g., after protobuf changes)");
        self.node_structure_cache.clear();
        self.save()
    }
}

pub struct ClaudeSession {
    conversation_id: Option<String>,
    forever: bool,
}

impl ClaudeSession {
    pub fn new(forever: bool) -> Self {
        Self {
            conversation_id: None,
            forever,
        }
    }

    pub fn call_claude(&mut self, prompt: &str, new_conversation: bool) -> Result<String> {
        use std::io::Write;
        use std::process::Stdio;

        let mut cmd = Command::new("claude");
        cmd.arg("--print") // Non-interactive mode
            .arg("--output-format")
            .arg("json")
            .arg("--permission-mode")
            .arg("acceptEdits")
            .arg("--allowedTools")
            .arg("Bash,Read,Write,Edit,MultiEdit,Grep,Glob,LS")
            .env_remove("ANTHROPIC_API_KEY") // Unset API key to use CLI auth
            .current_dir(project_root()) // Set working directory to monorepo root
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Reuse conversation if we have one and not explicitly starting new
        if !new_conversation && self.conversation_id.is_some() {
            if let Some(ref conv_id) = self.conversation_id {
                cmd.arg("--resume").arg(conv_id);
            }
        }

        eprintln!(
            "Calling Claude with prompt: {}",
            &prompt.lines().next().unwrap_or("")
        );

        // Spawn the process
        let mut child = cmd
            .spawn()
            .context("Failed to execute claude CLI. Make sure it's installed and in PATH")?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
            stdin.flush()?;
        }

        // Wait for completion and collect output
        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Check if this is a usage limit error
            if stderr.contains("usage limit") || stdout.contains("usage limit") {
                if self.forever {
                    eprintln!("Hit Claude API usage limit. Sleeping for 5 hours 30 minutes...");
                    thread::sleep(Duration::from_secs(5 * 3600 + 30 * 60)); // 5h 30m
                    eprintln!("Resuming after usage limit sleep");
                    // Retry the call after sleeping
                    return self.call_claude(prompt, new_conversation);
                }
            }

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
