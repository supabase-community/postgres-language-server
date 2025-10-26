# CLI Architecture Refactor Plan

## Current Problems

### 1. CommandRunner Trait Forces All Commands Through Same Pipeline
```rust
pub(crate) trait CommandRunner: Sized {
    fn run(&mut self, session: CliSession, cli_options: &CliOptions) -> Result<()> {
        // Forces: config loading → VCS setup → file traversal → reporting
        let (execution, paths) = self.configure_workspace(fs, console, workspace, cli_options)?;
        execute_mode(execution, session, cli_options, paths)
    }
}
```

**Issues:**
- Only `Check` command uses this trait
- Commands like `version`, `clean`, `init` are ad-hoc functions
- No way for commands to skip parts they don't need (e.g., dblint needs config but not traversal)
- Forces unnecessary complexity on simple commands

### 2. TraversalMode::Dummy Exists For Wrong Reasons
```rust
pub enum TraversalMode {
    Dummy,  // Only exists because commands are forced through traversal
    Check { stdin: Option<Stdin>, vcs_targeted: VcsTargeted },
}
```

Commands that don't need traversal shouldn't have a "dummy" mode - they just shouldn't traverse.

### 3. Execution Struct Conflates Concerns
```rust
pub struct Execution {
    report_mode: ReportMode,      // How to report (Terminal/GitHub/GitLab)
    traversal_mode: TraversalMode, // How to process files
    max_diagnostics: u32,
}
```

**Problem:** Bundles processing config with reporting config. Commands that don't traverse (like `dblint`) still need reporting but don't need traversal config.

### 4. execute_mode() is Monolithic
```rust
pub fn execute_mode(execution: Execution, session: CliSession, ...) -> Result<()> {
    // Does: stdin handling + traversal + reporting + exit codes
    // Can't be reused by commands that don't traverse
}
```

### 5. Scattered and Inconsistent Structure
- `commands/` has mix of trait impls and functions
- `execute/` has traversal logic tightly coupled to reporting
- No clear separation of workspace setup, execution, and reporting
- Commands repeat boilerplate for config loading, setup, reporting

---

## Proposed Clean Architecture

### Three Command Tiers

We have three distinct types of commands:

**Tier 1: Simple** - No workspace needed
- `version`, `clean`, `stop`, `print_socket`
- Just need Console/FileSystem

**Tier 2: Workspace-only** - Config/workspace, no file traversal
- `init`, `dblint`
- Load config, setup workspace, analyze, report

**Tier 3: Full Pipeline** - Config + workspace + file traversal
- `check`, (future: `format`, `lint`)
- Load config, setup workspace with VCS, traverse files, report

### New Architecture Layers

```
┌─────────────────────────────────────────┐
│  commands/                              │
│  Simple functions, no forced trait     │
│  - version, clean, init, dblint, check  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  CliSession with Helper Methods         │
│  - prepare_with_config()                │
│  - setup_workspace(VcsIntegration)      │
│  - report()                             │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  workspace.rs (NEW)                     │
│  - load_config()                        │
│  - setup_workspace(..., VcsIntegration) │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  execute/ (RESTRUCTURED)                │
│  - ExecutionConfig + helpers            │
│    - run_files()                        │
│    - run_stdin()                        │
│  Clear split between walker & stdin     │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  reporter/                              │
│  - Reporter struct + Report enum        │
│  Works with ANY diagnostic source       │
└─────────────────────────────────────────┘
```

---

## Detailed Design

### 1. CliSession Helper Methods

Add helper methods to reduce boilerplate:

```rust
// crates/pgt_cli/src/lib.rs

impl<'app> CliSession<'app> {
    /// Helper accessors
    pub fn fs(&self) -> &DynRef<'_, dyn FileSystem> {
        &self.app.fs
    }

    pub fn console(&mut self) -> &mut dyn Console {
        &mut *self.app.console
    }

    pub fn workspace(&self) -> &dyn Workspace {
        &*self.app.workspace
    }

    /// Common setup: logging + config loading + merging
    pub fn prepare_with_config(
        &mut self,
        cli_options: &CliOptions,
        cli_configuration: Option<PartialConfiguration>,
    ) -> Result<PartialConfiguration, CliDiagnostic> {
        setup_cli_subscriber(cli_options.log_level, cli_options.log_kind);

        // Take borrows once so immutable/mutable access do not overlap
        let fs = self.fs();
        let console = self.console();
        let loaded = workspace::load_config(
            fs,
            console,
            cli_options.as_configuration_path_hint(),
        )?;

        let mut configuration = loaded.configuration;
        if let Some(cli_config) = cli_configuration {
            configuration.merge_with(cli_config);
        }

        Ok(configuration)
    }

    /// Setup workspace with optional VCS integration
    pub fn setup_workspace(
        &mut self,
        configuration: PartialConfiguration,
        vcs: VcsIntegration,
    ) -> Result<(), CliDiagnostic> {
        let workspace = self.workspace();
        let fs = self.fs();

        workspace::setup_workspace(workspace, fs, configuration, vcs)
    }

    /// Report results (common final step)
    pub fn report(
        &mut self,
        cli_options: &CliOptions,
        report: Report,
    ) -> Result<(), CliDiagnostic> {
        let mut reporter = Reporter::from_cli_options(cli_options);
        reporter.report(self.console(), report)
    }
}

/// Controls whether workspace setup includes VCS integration
pub enum VcsIntegration {
    /// Enable VCS integration (gitignore, changed files detection)
    Enabled,
    /// Skip VCS integration
    Disabled,
}
}
```

### 2. Workspace Setup Layer (NEW)

```rust
// crates/pgt_cli/src/workspace.rs - NEW FILE

use crate::VcsIntegration;
use pgt_configuration::PartialConfiguration;
use pgt_console::{Console, ConsoleExt, markup};
use pgt_fs::{ConfigName, FileSystem};
use pgt_workspace::{DynRef, Workspace};
use pgt_workspace::configuration::{LoadedConfiguration, load_configuration};
use pgt_workspace::workspace::{RegisterProjectFolderParams, UpdateSettingsParams};
use std::path::PathBuf;

/// Load configuration from filesystem with deprecation warning
pub fn load_config(
    fs: &DynRef<'_, dyn FileSystem>,
    console: &mut dyn Console,
    config_hint: Option<&PathBuf>,
) -> Result<LoadedConfiguration, CliDiagnostic> {
    let loaded = load_configuration(fs, config_hint)?;

    if let Some(config_path) = &loaded.file_path {
        if let Some(file_name) = config_path.file_name().and_then(|n| n.to_str()) {
            if ConfigName::is_deprecated(file_name) {
                console.log(markup! {
                    <Warn>"Warning: "</Warn>"Deprecated config filename. Use 'postgres-language-server.jsonc'.\n"
                });
            }
        }
    }

    Ok(loaded)
}

pub fn setup_workspace(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: PartialConfiguration,
    vcs: VcsIntegration,
) -> Result<(), CliDiagnostic> {
    let (vcs_base_path, gitignore_matches) = match vcs {
        VcsIntegration::Enabled => configuration
            .retrieve_gitignore_matches(fs, fs.working_directory().as_deref())
            .map_err(CliDiagnostic::from)?,
        VcsIntegration::Disabled => (None, vec![]),
    };

    workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: fs.working_directory(),
            set_as_current_workspace: true,
        })
        .map_err(CliDiagnostic::from)?;

    workspace
        .update_settings(UpdateSettingsParams {
            workspace_directory: fs.working_directory(),
            configuration,
            vcs_base_path,
            gitignore_matches,
        })
        .map_err(CliDiagnostic::from)?;

    Ok(())
}
```

### 3. execute/ Module (restructured, not new)

We keep the familiar `execute/` name but make the responsibilities crystal clear:

```text
execute/
  mod.rs         - public entry points (run_files, run_stdin) + ExecutionConfig
  config.rs      - ExecutionMode / FixMode / VcsTargeting helpers
  walk.rs        - filesystem traversal + VCS-aware path filtering
  stdin.rs       - converts stdin payload into process_file jobs
  process_file/  - unchanged, per-file logic
```

#### Execution configuration lives in `execute/config.rs`

```rust
// Shared across files + stdin paths
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub limits: ExecutionLimits,
}

pub enum ExecutionMode {
    Check { fix_mode: Option<FixMode>, vcs: VcsTargeting },
    Format { write: bool, ignore_errors: bool, vcs: VcsTargeting },
    Lint { only: Vec<RuleSelector>, skip: Vec<RuleSelector>, fix_mode: Option<FixMode>, vcs: VcsTargeting },
}

pub enum FixMode {
    Safe,
    Suggested,
}

pub struct VcsTargeting {
    pub staged: bool,
    pub changed: bool,
}

pub struct ExecutionLimits {
    pub max_diagnostics: u32,
    pub allow_writes: bool,
}

impl ExecutionConfig {
    pub fn new(mode: ExecutionMode, max_diagnostics: u32) -> Self {
        let allow_writes = matches!(
            &mode,
            ExecutionMode::Check { fix_mode: Some(_), .. }
                | ExecutionMode::Format { write: true, .. }
                | ExecutionMode::Lint { fix_mode: Some(_), .. }
        );
        Self {
            mode,
            limits: ExecutionLimits {
                max_diagnostics,
                allow_writes,
            },
        }
    }
}
```

Having a single config object means commands share one type regardless of whether they operate on stdin or directories. `allow_writes` is derived from the mode so CLI commands can gate destructive actions early.

#### Public helpers stay tiny

```rust
pub fn run_files(
    session: &mut CliSession,
    config: &ExecutionConfig,
    paths: Vec<OsString>,
) -> Result<ExecutionSummary, CliDiagnostic> {
    walk::traverse(session, config, paths)
}

pub fn run_stdin(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<ExecutionSummary, CliDiagnostic> {
    stdin::process(session, config, payload)
}

pub struct StdinPayload {
    pub path: PathBuf,
    pub content: String,
}
```

Where `StdinPayload` is a simple `{ path: PathBuf, content: String }` struct defined next to the helpers.

#### Reporting structures stay outside execute/

`execute/` no longer defines the old `DiagnosticResult` / `TraversalResult` structs. Instead it emits neutral `ExecutionSummary` values (counts, changed/skipped, collected diagnostics). The reusable `DiagnosticSummary` type that dblint also needs lives under `reporter/summary.rs` (see next section). This keeps execute/ focused on producing data, not on how it will be displayed.

```rust
// crates/pgt_cli/src/reporter/mod.rs - ADD REPORT CONFIG

/// Configuration for how to report results
/// SEPARATE from traversal configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub mode: ReportMode,
    pub verbose: bool,
    pub diagnostic_level: DiagnosticLevel,
    pub error_on_warnings: bool,
    pub no_errors_on_unmatched: bool,
}

impl ReportConfig {
    pub fn from_cli_options(cli_options: &CliOptions) -> Self {
        Self {
            mode: cli_options.reporter.clone().into(),
            verbose: cli_options.verbose,
            diagnostic_level: cli_options.diagnostic_level,
            error_on_warnings: cli_options.error_on_warnings,
            no_errors_on_unmatched: cli_options.no_errors_on_unmatched,
        }
    }
}

/// Summaries live with the reporter so non-traversal commands (dblint) can reuse them
pub struct DiagnosticSummary {
    pub diagnostics: Vec<Error>,
    pub duration: Duration,
    pub errors: u32,
    pub warnings: u32,
}

impl DiagnosticSummary {
    pub fn from_diagnostics(diagnostics: Vec<Error>, duration: Duration) -> Self {
        let errors = diagnostics.iter().filter(|d| d.severity().is_error()).count() as u32;
        let warnings = diagnostics.iter().filter(|d| d.severity().is_warning()).count() as u32;
        Self { diagnostics, duration, errors, warnings }
    }
}

pub struct ExecutionSummary {
    pub diagnostics: DiagnosticSummary,
    pub evaluated_paths: Vec<PathBuf>,
    pub changed: usize,
    pub unchanged: usize,
    pub skipped: usize,
}

/// User-facing type describing what should be reported
pub enum Report {
    Diagnostics {
        summary: DiagnosticSummary,
        command_name: &'static str,
    },
    Traversal {
        summary: ExecutionSummary,
        command_name: &'static str,
    },
}

impl Report {
    pub fn from_diagnostic_summary(
        command_name: &'static str,
        summary: DiagnosticSummary,
    ) -> Self {
        Report::Diagnostics { summary, command_name }
    }

    pub fn from_execution_summary(
        command_name: &'static str,
        summary: ExecutionSummary,
    ) -> Self {
        Report::Traversal { summary, command_name }
    }
}

/// Reporter struct instead of bare function calls
pub struct Reporter {
    config: ReportConfig,
}

impl Reporter {
    pub fn from_cli_options(cli_options: &CliOptions) -> Self {
        Self {
            config: ReportConfig::from_cli_options(cli_options),
        }
    }

    pub fn report(
        &mut self,
        console: &mut dyn Console,
        report: Report,
    ) -> Result<(), CliDiagnostic> {
        match report {
            Report::Diagnostics { summary, command_name } => {
                self.report_diagnostics(console, summary, command_name)
            }
            Report::Traversal { summary, command_name } => {
                self.report_traversal(console, summary, command_name)
            }
        }
    }

    fn report_diagnostics(
        &self,
        console: &mut dyn Console,
        summary: DiagnosticSummary,
        command_name: &str,
    ) -> Result<(), CliDiagnostic> {
        // existing logic from report_analysis
    }

    fn report_traversal(
        &self,
        console: &mut dyn Console,
        summary: ExecutionSummary,
        command_name: &str,
    ) -> Result<(), CliDiagnostic> {
        // existing logic from report_traversal
    }
}

// Implementation note: the bodies of `report_diagnostics` and `report_traversal`
// reuse today's `report_analysis` / `report_traversal` logic verbatim, just moved
// behind the struct methods so callers always use the same entry point.
```

### 4. Example Commands

#### Tier 1: Simple Command (No Config)

```rust
// crates/pgt_cli/src/commands/version.rs

pub fn version(mut session: CliSession) -> Result<(), CliDiagnostic> {
    session.console().log(markup! {
        "CLI:        "{VERSION}
    });

    match session.workspace().server_info() {
        None => {
            session.console().log(markup! {
                "Server:     "<Dim>"not connected"</Dim>
            });
        }
        Some(info) => {
            session.console().log(markup! {
                "Server:
                  Name:     "{info.name}"
                  Version:  "{info.version.unwrap_or_else(|| "-".to_string())}
            });
        }
    };

    Ok(())
}
```

#### Tier 2: Config-Only Command (No Traversal)

```rust
// crates/pgt_cli/src/commands/dblint.rs

pub fn dblint(
    mut session: CliSession,
    cli_options: &CliOptions,
    cli_configuration: Option<PartialConfiguration>,
) -> Result<(), CliDiagnostic> {
    // Step 1: Common setup (logging + config loading + merging)
    let configuration = session.prepare_with_config(cli_options, cli_configuration)?;

    // Step 2: Setup workspace (no VCS needed)
    session.setup_workspace(configuration.clone(), VcsIntegration::Disabled)?;

    // Step 3: Custom analysis logic (no file traversal!)
    let start = Instant::now();
    let diagnostics = analyze_workspace_config(session.workspace(), &configuration)?;
    let duration = start.elapsed();

    // Step 4: Build summary (clean - no fake traversal data!)
    let summary = DiagnosticSummary::from_diagnostics(diagnostics, duration);

    // Step 5: Report (same infrastructure as check)
    session.report(
        cli_options,
        Report::from_diagnostic_summary("dblint", summary),
    )
}

fn analyze_workspace_config(
    workspace: &dyn Workspace,
    config: &PartialConfiguration,
) -> Result<Vec<Error>, CliDiagnostic> {
    // Your dblint analysis logic here
    // Example: validate linting rules, check for conflicts, etc.
    let mut diagnostics = vec![];

    if let Some(linter_config) = &config.linter {
        if linter_config.rules.is_empty() {
            diagnostics.push(
                Error::from_message("No linting rules configured")
                    .with_severity(Severity::Warning)
                    .with_category(category!("dblint"))
            );
        }
    }

    Ok(diagnostics)
}

```

#### Tier 3: Full Pipeline Command (Traversal)

```rust
// crates/pgt_cli/src/commands/check.rs

pub struct CheckArgs {
    pub configuration: Option<PartialConfiguration>,
    pub paths: Vec<OsString>,
    pub stdin_file_path: Option<String>,
    pub staged: bool,
    pub changed: bool,
    pub since: Option<String>,
    pub apply: bool,
    pub apply_unsafe: bool,
}

pub fn check(
    mut session: CliSession,
    cli_options: &CliOptions,
    args: CheckArgs,
) -> Result<(), CliDiagnostic> {
    // Step 1: Common setup
    let configuration = session.prepare_with_config(cli_options, args.configuration)?;
    validate_args(&args)?;

    // Step 2: Setup workspace with VCS (needed for traversal)
    session.setup_workspace(configuration.clone(), VcsIntegration::Enabled)?;

    // Step 3: Compute paths (from CLI args, VCS, or default)
    let paths = compute_paths(session.fs(), &configuration, &args)?;

    // Step 4: Build execution mode
    let mode = ExecutionMode::Check {
        fix_mode: if args.apply_unsafe {
            Some(FixMode::Suggested)
        } else if args.apply {
            Some(FixMode::Safe)
        } else {
            None
        },
        vcs: VcsTargeting {
            staged: args.staged,
            changed: args.changed,
        },
    };

    // Step 5: Build config once and reuse it regardless of the input source
    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics
    } else {
        u32::MAX
    };
    let execution_config = ExecutionConfig::new(mode, max_diagnostics);

    let summary = if let Some(stdin_path) = args.stdin_file_path {
        let content = session.console().read()
            .ok_or_else(|| CliDiagnostic::missing_argument("stdin", "check"))?;

        execute::run_stdin(
            &mut session,
            &execution_config,
            StdinPayload {
                path: PathBuf::from(stdin_path),
                content,
            },
        )?
    } else {
        execute::run_files(&mut session, &execution_config, paths)?
    };

    // Step 6: Report
    session.report(
        cli_options,
        Report::from_execution_summary("check", summary),
    )
}

fn validate_args(args: &CheckArgs) -> Result<(), CliDiagnostic> {
    if args.since.is_some() && !args.changed {
        return Err(CliDiagnostic::incompatible_arguments("since", "changed"));
    }
    if args.changed && args.staged {
        return Err(CliDiagnostic::incompatible_arguments("changed", "staged"));
    }
    Ok(())
}

fn compute_paths(
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: &PartialConfiguration,
    args: &CheckArgs,
) -> Result<Vec<OsString>, CliDiagnostic> {
    if args.changed {
        get_changed_files(fs, configuration, args.since.as_deref())
    } else if args.staged {
        get_staged_files(fs)
    } else if !args.paths.is_empty() {
        Ok(args.paths.clone())
    } else {
        Ok(vec![]) // Default to current directory (handled inside execute::run_files)
    }
}
```

### 5. Execution Flow

Here's how the "full pipeline" path works in the check command:

```
check command
    ↓
execute::run_files(&mut session, &config, paths)
    ↓
walk::traverse(session, config, paths)
    ↓
Walk file system from starting paths
    ↓
For each file found:
    ↓
process_file::process_file(options, path)
    ↓  (dispatches based on mode)
    ↓
For Check mode:
  workspace.pull_diagnostics() - runs linting
For Format mode:
  workspace.format() - formats file
For Lint mode:
  workspace.pull_diagnostics() with specific rules
    ↓
Send diagnostics to collector thread
    ↓
After traversal completes:
    ↓
Return ExecutionSummary { diagnostics, counts }
    ↓
check command → Reporter::report(...)
```

**Key insight:** The `execute/` module is **mode-agnostic** - it works with Check, Format, Lint, etc. Commands like `dblint`, `version`, `init` never import it because they don't process files.

### 6. Updated lib.rs

```rust
// crates/pgt_cli/src/lib.rs

mod cli_options;
mod commands;
mod diagnostics;
mod logging;
mod reporter;
mod execute;    // RESTRUCTURED - replaces old execute/
mod workspace;  // NEW

impl<'app> CliSession<'app> {
    // ... helper methods defined above ...

    pub fn run(self, command: PgtCommand) -> Result<(), CliDiagnostic> {
        match command {
            PgtCommand::Version(_) => commands::version::version(self),
            PgtCommand::Clean => commands::clean::clean(self),
            PgtCommand::Init => commands::init::init(self),

            PgtCommand::Dblint { cli_options, configuration } => {
                commands::dblint::dblint(self, &cli_options, configuration)
            }

            PgtCommand::Check {
                cli_options,
                configuration,
                paths,
                stdin_file_path,
                staged,
                changed,
                since,
            } => {
                commands::check::check(
                    self,
                    &cli_options,
                    commands::check::CheckArgs {
                        configuration,
                        paths,
                        stdin_file_path,
                        staged,
                        changed,
                        since,
                        apply: false, // Add these flags to PgtCommand enum
                        apply_unsafe: false,
                    },
                )
            }

            // ... other commands
        }
    }
}

// REMOVE: run_command() helper
// REMOVE: CommandRunner trait
```

---

## Comparison: Before vs After

| Aspect | Current | Proposed |
|--------|---------|----------|
| **Command structure** | Mix of CommandRunner trait + ad-hoc functions | All functions, no forced trait |
| **Simple commands** | Ad-hoc implementations | Clean functions using helpers |
| **Config-only commands** | No good pattern | `prepare_with_config()` + `setup_workspace(VcsIntegration::Disabled)` |
| **Workspace setup** | `setup_full_workspace()` vs `setup_minimal_workspace()` (unclear) | `setup_workspace(config, VcsIntegration::Enabled/Disabled)` (clear) |
| **Result types** | TraversalSummary used even when not traversing | `DiagnosticSummary` for analysis, `ExecutionSummary` for traversal |
| **Traversal commands** | Forced through CommandRunner | Explicit `execute::run_files()` call |
| **Code reuse** | Trait with mandatory methods | Helper methods on CliSession |
| **Traversal modes** | Has Dummy mode | No Dummy - traversal is opt-in |
| **Execution bundle** | Conflates processing + reporting | Split: ExecutionConfig + ReportConfig |
| **Reporting** | Coupled to execute_mode | `Reporter` struct with `Report` payloads |
| **Boilerplate** | High - repeated config loading | Low - helpers reduce repetition |
| **Flexibility** | Low - forced through pipeline | High - compose what you need |
| **Clarity** | Unclear what commands need | Crystal clear from function calls |

---

## Pros and Cons

### Pros

✅ **Clear command tiers** - Easy to understand what each command needs
✅ **No forced abstractions** - Commands use only what they need
✅ **ExecutionMode preserved** - Keeps rich configuration like Biome
✅ **No Dummy mode** - Traversal is truly optional
✅ **Separation of concerns** - Processing config separate from reporting config
✅ **Reusable infrastructure** - Reporting works for all commands
✅ **Reduced boilerplate** - Helper methods eliminate repetition
✅ **Extensible** - Easy to add new modes (Format, Lint) or commands
✅ **Testable** - Each layer can be tested independently
✅ **Explicit data flow** - No hidden trait magic
✅ **Flexible reporting** - Any command can generate diagnostics and use same reporting
✅ **Clear workspace setup** - `VcsIntegration::Enabled/Disabled` is self-documenting
✅ **Proper result types** - DiagnosticSummary vs ExecutionSummary - no fake data

### Cons

⚠️ **More methods on CliSession** - Could become bloated if too many helpers added
⚠️ **No enforced structure** - Commands could skip important steps if not careful
⚠️ **Duplication risk** - Without trait, commands might duplicate logic (mitigated by helpers)
⚠️ **Learning curve** - New contributors need to understand helper methods
⚠️ **No compile-time guarantees** - Can't enforce "must setup workspace before traversal"

### Mitigation Strategies

- Keep CliSession helpers focused on truly common patterns
- Document command implementation patterns clearly
- Use helper function modules (like workspace.rs) for shared logic
- Add integration tests that verify correct command flow
- Add focused helper functions (e.g., `ensure_workspace_ready`) if we need stronger guidance later

---

## Migration Path

### Phase 1: Setup Foundation
1. Create `crates/pgt_cli/src/workspace.rs` with helper functions
2. Add helper methods to `CliSession` (accessors + borrow-safe `prepare_with_config` + `setup_workspace`)
3. Split `Execution` into `ExecutionConfig` and `ReportConfig`
4. Introduce `Reporter` struct (`from_cli_options` + `report()`)

### Phase 2: Migrate Simple Commands
1. Refactor `version` to use new helpers (already simple)
2. Refactor `clean` to use new helpers
3. Smoke-test both commands

### Phase 3: Add Dblint (Config-Only Command)
1. Implement `commands/dblint.rs` using `prepare_with_config()` + `setup_workspace(...Disabled)`
2. Implement `analyze_workspace_config()` logic
3. Add dblint to `PgtCommand` enum and `CliSession::run()`
4. Test dblint command

### Phase 4: Migrate Check Command
1. Restructure `execute/` (config.rs, walk.rs, stdin.rs, process_file/)
2. Add `execute::run_files()` helper (thin wrapper over walk)
3. Add `execute::run_stdin()` helper (buffer path)
4. Update `walk.rs` to use `ExecutionConfig`
5. Ensure walker no longer depends on stdin specifics
6. Refactor `check.rs` to call `run_stdin`/`run_files`
7. Test check command thoroughly (traversal, stdin, VCS modes)

### Phase 5: Cleanup
1. Remove `CommandRunner` trait from `commands/mod.rs`
2. Remove `execute_mode()` function
3. Remove `TraversalMode::Dummy`
4. Remove old `Execution` struct
5. Update imports to new `execute::{self, config, stdin}` layout
6. Update/extend tests
7. Update documentation

### Phase 6: Future Commands
1. Add `format` command using ExecutionMode::Format
2. Add `lint` command using ExecutionMode::Lint
3. Each new command follows established patterns

---

## File Changes Summary

### Files to Create/Rename
- `crates/pgt_cli/src/workspace.rs` - NEW - Workspace setup helpers
- `crates/pgt_cli/src/commands/dblint.rs` - NEW - Dblint command

### Files to Modify (Significant)
- `crates/pgt_cli/src/lib.rs` - Add helper methods to CliSession, simplify run()
- `crates/pgt_cli/src/execute/mod.rs` - Export `ExecutionConfig`, `run_files`, `run_stdin`
- `crates/pgt_cli/src/execute/config.rs` - NEW internal module for modes/limits
- `crates/pgt_cli/src/execute/walk.rs` - Rewritten to use `ExecutionConfig`
- `crates/pgt_cli/src/execute/stdin.rs` - Buffer path feeding process_file
- `crates/pgt_cli/src/reporter/mod.rs` - Add ReportConfig, `Reporter` struct, `Report` enum, summaries
- `crates/pgt_cli/src/commands/check.rs` - Call `run_stdin`/`run_files`
- `crates/pgt_cli/src/commands/mod.rs` - Remove CommandRunner trait (~135 lines)
- `crates/pgt_cli/src/execute/process_file/mod.rs` - Dispatch based on ExecutionMode

### Files to Modify (Minor)
- `crates/pgt_cli/src/commands/version.rs` - Use helper methods
- `crates/pgt_cli/src/commands/clean.rs` - Use helper methods
- `crates/pgt_cli/src/execute/process_file/*.rs` - Use ExecutionMode from config
- Remove the old `execute/std_in.rs` and fold logic into `execute/stdin.rs`

### Lines of Code Impact
- **Delete:** ~200 lines (CommandRunner trait, execute_mode monolith, Dummy mode)
- **Add:** ~300 lines (workspace.rs, helpers, dblint, configs)
- **Net:** +100 lines but much clearer structure

---

## Key Design Principles

1. **Opt-in over forced** - Commands choose what they need, not forced through pipeline
2. **Separation of concerns** - Config, traversal, reporting are independent
3. **Explicit over implicit** - Function calls show what's happening, no hidden trait magic
4. **Simple stays simple** - Don't force complexity on commands that don't need it
5. **Reusable infrastructure** - Reporting works for any diagnostic source
6. **Composable helpers** - Small, focused helper methods instead of monolithic abstractions
7. **Rich configuration** - ExecutionMode captures "how" to process files, keep it flexible
8. **Clear boundaries** - `execute/` owns runtime pipeline, stdin/files are explicit helpers

### Why keep `execute/` but reshape it?

**Original pain:** `execute/` mixed config, traversal, stdin, and reporting glue. You had to understand everything to touch anything.

**New structure:**
```
execute/
  mod.rs        - `ExecutionConfig`, `run_files`, `run_stdin`
  config.rs     - ExecutionMode + FixMode + limits
  walk.rs       - directory walking + VCS targeting
  stdin.rs      - buffer path, no filesystem assumptions
  process_file/ - unchanged processing logic
```

Now the name still matches the CLI command (`pgt execute check`), but the responsibilities are sliced: commands talk to `run_*`, the walker never sees stdin, and reporting lives elsewhere.

---

## Open Questions

1. **Should we add more optional helpers?** e.g., `compute_vcs_paths()`, `read_stdin()`
   - Lean towards yes, but only after seeing repeated patterns

2. **Should ReportConfig handle category name or commands pass it?**
   - Currently commands pass command_name - keeps reporting generic

3. **Do we need a WorkspaceConfig struct?**
   - Currently passing PartialConfiguration directly - works but could bundle with paths

4. **Should stdin handling stay inside execute/?**
   - Current plan keeps `run_stdin()` next to `run_files()` for shared logic, but we can split later if it grows complex

---

## Success Criteria

✅ All three command tiers have clear, distinct implementations
✅ No TraversalMode::Dummy
✅ Dblint command works without traversal
✅ Check command works with traversal (files, stdin, VCS modes)
✅ All reporters (Terminal, GitHub, GitLab, JUnit) work for all commands
✅ Less boilerplate in command implementations
✅ All existing tests pass
✅ Code is easier to understand for new contributors

---

## References

- Biome CLI architecture - Good example of rich execution modes
- Current `pgt_cli` implementation - Understanding existing patterns
- Rust API guidelines - General API ergonomics and error handling guidance
