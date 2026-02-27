use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
#[cfg(unix)]
use pgls_cli::SocketTransport;
#[cfg(unix)]
use pgls_workspace::workspace::{TransportRequest, WorkspaceTransport};
use serde_json::Value;
#[cfg(unix)]
use tokio::net::UnixStream;
use xshell::Shell;

use crate::flags;

const DEFAULT_ITERATIONS: usize = 200;
const DEFAULT_PAUSE_MS: u64 = 20;

impl flags::LeakCheck {
    pub(crate) fn run(self, _sh: &Shell) -> anyhow::Result<()> {
        if !cfg!(target_os = "macos") {
            bail!("`xtask leak-check` is currently implemented only for macOS (`leaks` tool).");
        }

        if Command::new("leaks").arg("--help").output().is_err() {
            bail!("`leaks` not found — install Xcode Command Line Tools (`xcode-select --install`)");
        }

        let iterations = self.iterations.unwrap_or(DEFAULT_ITERATIONS);
        let pause = Duration::from_millis(self.pause_ms.unwrap_or(DEFAULT_PAUSE_MS));
        let probe = self.probe.unwrap_or_else(|| "lsp".to_string());

        match probe.as_str() {
            "lsp" => run_lsp_probe(iterations, pause),
            "cli-timeout" => run_cli_timeout_probe(iterations),
            "both" => {
                run_lsp_probe(iterations, pause)?;
                run_cli_timeout_probe(iterations)
            }
            other => bail!("invalid --probe value `{other}` (expected: lsp | cli-timeout | both)"),
        }
    }
}

fn run_lsp_probe(iterations: usize, pause: Duration) -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("pgls_cli")
        .status()
        .context("failed to execute cargo build for pgls_cli")?;
    if !status.success() {
        bail!("failed to build pgls_cli binary");
    }

    let root = std::env::current_dir().context("failed to get current directory")?;
    let binary = root.join("target/debug/postgres-language-server");

    if !binary.exists() {
        bail!("binary not found at {}", binary.display());
    }

    let server = ProcessGuard::spawn(
        Command::new(&binary)
            .arg("__run_server")
            .arg("--stop-on-disconnect")
            .arg("--log-level=error")
            .arg("--log-kind=hierarchical")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
        "server",
    )?;

    wait_for_socket(server.pid())?;

    let mut proxy = ProcessGuard::spawn(
        Command::new(&binary)
            .arg("lsp-proxy")
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()),
        "lsp-proxy",
    )?;

    let mut proxy_stdin = proxy
        .stdin()
        .context("failed to capture lsp-proxy stdin pipe")?;
    proxy.start_stdout_drain()?;

    run_lsp_churn(&mut proxy_stdin, iterations, pause)?;

    // Give background diagnostics/tasks a chance to settle.
    thread::sleep(Duration::from_millis(300));

    let leaks_output = run_leaks(server.pid())?;
    let pass = leaks_output
        .to_lowercase()
        .contains("0 leaks for 0 total leaked bytes");

    // Ask LSP to shutdown before cleanup. Ignore failures, cleanup guard is authoritative.
    let _ = send_shutdown_and_exit(&mut proxy_stdin);
    drop(proxy_stdin);

    if pass {
        println!(
            "LEAK_CHECK[LSP]: PASS (pid={}, iterations={iterations})",
            server.pid()
        );
        return Ok(());
    }

    println!("LEAK_CHECK[LSP]: FAIL (pid={})", server.pid());
    println!("{leaks_output}");
    bail!("leaks reported potential leaked allocations in lsp probe");
}

#[cfg(unix)]
fn run_cli_timeout_probe(iterations: usize) -> anyhow::Result<()> {
    let rss_start = current_rss_kb(std::process::id())?;

    let runtime = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
    let _enter = runtime.enter();
    let (stream_a, stream_b) =
        UnixStream::pair().context("failed to create unix socket pair for timeout probe")?;
    drop(stream_b);

    let (read, write) = stream_a.into_split();
    let transport =
        SocketTransport::open_with_timeout(runtime, read, write, Duration::from_millis(2));

    let mut channel_closed = 0usize;
    let mut timed_out = 0usize;
    let mut other_errors = 0usize;

    for i in 0..iterations {
        let request = TransportRequest {
            id: i as u64,
            method: "pgls/get_file_content",
            params: (),
        };

        let result: Result<Value, _> = transport.request(request);
        match result {
            Err(pgls_workspace::TransportError::ChannelClosed) => channel_closed += 1,
            Err(pgls_workspace::TransportError::Timeout) => timed_out += 1,
            Err(_) => other_errors += 1,
            Ok(_) => {}
        }
    }

    // Keep transport alive until after RSS sampling so retained map growth is visible.
    let rss_end = current_rss_kb(std::process::id())?;
    let rss_delta = rss_end.saturating_sub(rss_start);

    println!(
        "LEAK_CHECK[CLI_TIMEOUT]: requests={iterations} channel_closed={channel_closed} timed_out={timed_out} other_errors={other_errors} rss_start_kb={rss_start} rss_end_kb={rss_end} rss_delta_kb={rss_delta}"
    );

    // Heuristic threshold for a strong "likely leak/retention" signal.
    let fail_threshold_kb: u64 = 20_000;
    if rss_delta >= fail_threshold_kb {
        bail!(
            "CLI timeout probe shows strong retained-memory growth (delta={rss_delta} KB >= {fail_threshold_kb} KB)"
        );
    }

    Ok(())
}

#[cfg(not(unix))]
fn run_cli_timeout_probe(_iterations: usize) -> anyhow::Result<()> {
    bail!("cli-timeout probe requires unix (UnixStream)");
}

struct ProcessGuard {
    child: Child,
    name: &'static str,
    stdout_drain: Option<thread::JoinHandle<()>>,
}

impl ProcessGuard {
    fn spawn(command: &mut Command, name: &'static str) -> anyhow::Result<Self> {
        let child = command
            .spawn()
            .with_context(|| format!("failed to spawn {name}"))?;
        Ok(Self {
            child,
            name,
            stdout_drain: None,
        })
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    fn start_stdout_drain(&mut self) -> anyhow::Result<()> {
        let stdout = self
            .child
            .stdout
            .take()
            .context("failed to capture child stdout")?;

        let handle = thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            let mut sink = std::io::sink();
            let _ = std::io::copy(&mut reader, &mut sink);
        });

        self.stdout_drain = Some(handle);
        Ok(())
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(handle) = self.stdout_drain.take() {
            let _ = handle.join();
        }
        let _ = self.name;
    }
}

fn wait_for_socket(pid: u32) -> anyhow::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let output = Command::new("lsof")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .context("failed to run `lsof` while waiting for socket")?;

        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        if combined.contains("pgls-socket-") {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }

    bail!("timed out waiting for server socket to become ready");
}

fn run_lsp_churn(stdin: &mut ChildStdin, iterations: usize, pause: Duration) -> anyhow::Result<()> {
    let uri = "file:///tmp/pgls-leak-check.sql";

    send_lsp_json(
        stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null}}),
    )?;
    send_lsp_json(
        stdin,
        serde_json::json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    )?;

    for i in 0..iterations {
        let open_text = format!("select {i} as value;");
        let changed_text = format!("select {i} as value, {} as extra;", i + 1);

        send_lsp_json(
            stdin,
            serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"sql","version":1,"text":open_text}}}),
        )?;

        send_lsp_json(
            stdin,
            serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":changed_text}]}}),
        )?;

        send_lsp_json(
            stdin,
            serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        )?;

        thread::sleep(pause);
    }

    Ok(())
}

fn send_shutdown_and_exit(stdin: &mut ChildStdin) -> anyhow::Result<()> {
    send_lsp_json(
        stdin,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
    )?;
    send_lsp_json(
        stdin,
        serde_json::json!({"jsonrpc":"2.0","method":"exit","params":null}),
    )?;
    Ok(())
}

fn send_lsp_json(stdin: &mut ChildStdin, value: Value) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&value).context("failed to serialize LSP message")?;
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    stdin
        .write_all(header.as_bytes())
        .context("failed to write LSP header")?;
    stdin
        .write_all(payload.as_bytes())
        .context("failed to write LSP payload")?;
    stdin.flush().context("failed to flush LSP payload")
}

fn run_leaks(pid: u32) -> anyhow::Result<String> {
    let output = Command::new("leaks")
        .arg(pid.to_string())
        .output()
        .context("failed to run `leaks`")?;

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(combined)
}

fn current_rss_kb(pid: u32) -> anyhow::Result<u64> {
    let output = Command::new("ps")
        .arg("-o")
        .arg("rss=")
        .arg("-p")
        .arg(pid.to_string())
        .output()
        .context("failed to run `ps` for rss sampling")?;

    let value = String::from_utf8_lossy(&output.stdout);
    let trimmed = value.trim();
    let rss = trimmed
        .parse::<u64>()
        .with_context(|| format!("failed to parse rss value from `{trimmed}`"))?;
    Ok(rss)
}
