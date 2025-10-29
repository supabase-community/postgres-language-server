use super::config::ExecutionConfig;
use super::process_file::{FileStatus, Message, process_file};
use crate::execute::diagnostics::PanicDiagnostic;
use crate::reporter::{Report, TraversalData};
use crate::{CliDiagnostic, CliSession};
use crossbeam::channel::{Receiver, Sender, unbounded};
use pgls_diagnostics::{DiagnosticExt, Error, Resource};
use pgls_fs::{FileSystem, PathInterner, PgLSPath};
use pgls_fs::{TraversalContext, TraversalScope};
use pgls_workspace::dome::Dome;
use pgls_workspace::workspace::IsPathIgnoredParams;
use pgls_workspace::{Workspace, WorkspaceError};
use rustc_hash::FxHashSet;
use std::collections::BTreeSet;
use std::sync::RwLock;
use std::sync::atomic::AtomicU32;
use std::{
    env::current_dir,
    ffi::OsString,
    panic::catch_unwind,
    path::PathBuf,
    sync::{
        Once,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub(crate) fn traverse(
    session: &mut CliSession,
    config: &ExecutionConfig,
    mut inputs: Vec<OsString>,
) -> Result<Report, CliDiagnostic> {
    init_thread_pool();

    if inputs.is_empty() && !config.mode.vcs().changed && !config.mode.vcs().staged {
        match current_dir() {
            Ok(current_dir) => inputs.push(current_dir.into_os_string()),
            Err(err) => return Err(CliDiagnostic::io_error(err)),
        }
    }

    let (interner, recv_files) = PathInterner::new();
    let (sender, receiver) = unbounded();

    let changed = AtomicUsize::new(0);
    let unchanged = AtomicUsize::new(0);
    let matches = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let fs = &**session.fs();
    let workspace = session.workspace();

    let max_diagnostics = config.max_diagnostics();
    let remaining_diagnostics = AtomicU32::new(max_diagnostics);

    let printer = DiagnosticsPrinter::new(config).with_max_diagnostics(max_diagnostics);

    let (duration, evaluated_paths, diagnostics) = thread::scope(|s| {
        let handler = thread::Builder::new()
            .name(String::from("pgt::console"))
            .spawn_scoped(s, || printer.run(receiver, recv_files))
            .expect("failed to spawn console thread");

        // The traversal context is scoped to ensure all the channels it
        // contains are properly closed once the traversal finishes
        let (elapsed, evaluated_paths) = traverse_inputs(
            fs,
            inputs,
            &TraversalOptions {
                fs,
                workspace,
                config,
                interner,
                matches: &matches,
                changed: &changed,
                unchanged: &unchanged,
                skipped: &skipped,
                messages: sender,
                remaining_diagnostics: &remaining_diagnostics,
                evaluated_paths: RwLock::default(),
            },
        );
        // wait for the main thread to finish
        let diagnostics = handler.join().unwrap();

        (elapsed, evaluated_paths, diagnostics)
    });

    let changed = changed.load(Ordering::Relaxed);
    let unchanged = unchanged.load(Ordering::Relaxed);
    let matches = matches.load(Ordering::Relaxed);
    let skipped = skipped.load(Ordering::Relaxed);
    let suggested_fixes_skipped = printer.skipped_fixes();
    let diagnostics_not_printed = printer.not_printed_diagnostics();

    let traversal = TraversalData {
        evaluated_paths,
        changed,
        unchanged,
        matches,
        skipped,
        suggested_fixes_skipped,
        diagnostics_not_printed,
        workspace_root: session.fs().working_directory(),
    };

    Ok(Report::new(
        diagnostics,
        duration,
        diagnostics_not_printed,
        Some(traversal),
    ))
}

/// This function will setup the global Rayon thread pool the first time it's called
///
/// This is currently only used to assign friendly debug names to the threads of the pool
fn init_thread_pool() {
    static INIT_ONCE: Once = Once::new();
    INIT_ONCE.call_once(|| {
        rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("pgt::worker_{index}"))
            .build_global()
            .expect("failed to initialize the global thread pool");
    });
}

/// Initiate the filesystem traversal tasks with the provided input paths and
/// run it to completion, returning the duration of the process and the evaluated paths
fn traverse_inputs(
    fs: &dyn FileSystem,
    inputs: Vec<OsString>,
    ctx: &TraversalOptions,
) -> (Duration, BTreeSet<PgLSPath>) {
    let start = Instant::now();
    fs.traversal(Box::new(move |scope: &dyn TraversalScope| {
        for input in inputs {
            scope.evaluate(ctx, PathBuf::from(input));
        }
    }));

    let paths = ctx.evaluated_paths();
    let dome = Dome::new(paths);
    let mut iter = dome.iter();
    fs.traversal(Box::new(|scope: &dyn TraversalScope| {
        while let Some(path) = iter.next_config() {
            scope.handle(ctx, path.to_path_buf());
        }

        for path in iter {
            scope.handle(ctx, path.to_path_buf());
        }
    }));

    (start.elapsed(), ctx.evaluated_paths())
}

// struct DiagnosticsReporter<'ctx> {}

struct DiagnosticsPrinter<'ctx> {
    _config: &'ctx ExecutionConfig,
    /// The maximum number of diagnostics the console thread is allowed to print
    max_diagnostics: u32,
    remaining_diagnostics: AtomicU32,
    /// Count of diagnostics that exceeded max_diagnostics and weren't printed
    not_printed_diagnostics: AtomicU32,
    printed_diagnostics: AtomicU32,
    total_skipped_suggested_fixes: AtomicU32,
}

impl<'ctx> DiagnosticsPrinter<'ctx> {
    fn new(config: &'ctx ExecutionConfig) -> Self {
        Self {
            _config: config,
            max_diagnostics: 20,
            remaining_diagnostics: AtomicU32::new(0),
            not_printed_diagnostics: AtomicU32::new(0),
            printed_diagnostics: AtomicU32::new(0),
            total_skipped_suggested_fixes: AtomicU32::new(0),
        }
    }

    fn with_max_diagnostics(mut self, value: u32) -> Self {
        self.max_diagnostics = value;
        self
    }

    fn not_printed_diagnostics(&self) -> u32 {
        self.not_printed_diagnostics.load(Ordering::Relaxed)
    }

    fn skipped_fixes(&self) -> u32 {
        self.total_skipped_suggested_fixes.load(Ordering::Relaxed)
    }

    /// Count the diagnostic, and then returns a boolean that tells if it should be printed
    fn should_store(&self) -> bool {
        let printed_diagnostics = self.printed_diagnostics.load(Ordering::Relaxed);
        let should_print = printed_diagnostics < self.max_diagnostics;
        if should_print {
            self.printed_diagnostics.fetch_add(1, Ordering::Relaxed);
            self.remaining_diagnostics.store(
                self.max_diagnostics.saturating_sub(printed_diagnostics),
                Ordering::Relaxed,
            );
        } else {
            self.not_printed_diagnostics.fetch_add(1, Ordering::Relaxed);
        }

        should_print
    }

    fn run(&self, receiver: Receiver<Message>, interner: Receiver<PathBuf>) -> Vec<Error> {
        let mut paths: FxHashSet<String> = FxHashSet::default();
        let mut diagnostics = vec![];

        while let Ok(msg) = receiver.recv() {
            match msg {
                Message::SkippedFixes {
                    skipped_suggested_fixes,
                } => {
                    self.total_skipped_suggested_fixes
                        .fetch_add(skipped_suggested_fixes, Ordering::Relaxed);
                }
                Message::Failure => {}
                Message::Error(mut err) => {
                    if let Some(Resource::File(file_path)) = err.location().resource.as_ref() {
                        let file_name = match paths.get(*file_path) {
                            Some(path) => Some(path),
                            None => loop {
                                match interner.recv() {
                                    Ok(path) => {
                                        paths.insert(path.display().to_string());
                                        if path.display().to_string() == *file_path {
                                            break paths.get(&path.display().to_string());
                                        }
                                    }
                                    Err(_) => break None,
                                }
                            },
                        };

                        if let Some(path) = file_name {
                            err = err.with_file_path(path.as_str());
                        }
                    }

                    if self.should_store() {
                        diagnostics.push(err);
                    }
                }
                Message::Diagnostics {
                    name,
                    content,
                    diagnostics: diag_list,
                    skipped_diagnostics,
                } => {
                    self.not_printed_diagnostics
                        .fetch_add(skipped_diagnostics, Ordering::Relaxed);

                    for diag in diag_list {
                        if self.should_store() {
                            let diag = diag.with_file_path(&name).with_file_source_code(&content);
                            diagnostics.push(diag);
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Context object shared between directory traversal tasks
pub(crate) struct TraversalOptions<'ctx, 'app> {
    /// Shared instance of [FileSystem]
    pub(crate) fs: &'app dyn FileSystem,
    /// Instance of [Workspace] used by this instance of the CLI
    pub(crate) workspace: &'ctx dyn Workspace,
    /// Determines how the files should be processed
    pub(crate) config: &'ctx ExecutionConfig,
    /// File paths interner cache used by the filesystem traversal
    interner: PathInterner,
    /// Shared atomic counter storing the number of changed files
    changed: &'ctx AtomicUsize,
    /// Shared atomic counter storing the number of unchanged files
    unchanged: &'ctx AtomicUsize,
    /// Shared atomic counter storing the number of unchanged files
    matches: &'ctx AtomicUsize,
    /// Shared atomic counter storing the number of skipped files
    skipped: &'ctx AtomicUsize,
    /// Channel sending messages to the display thread
    pub(crate) messages: Sender<Message>,
    /// The approximate number of diagnostics the console will print before
    /// folding the rest into the "skipped diagnostics" counter
    pub(crate) remaining_diagnostics: &'ctx AtomicU32,

    /// List of paths that should be processed
    pub(crate) evaluated_paths: RwLock<BTreeSet<PgLSPath>>,
}

impl TraversalOptions<'_, '_> {
    pub(crate) fn increment_changed(&self, path: &PgLSPath) {
        self.changed.fetch_add(1, Ordering::Relaxed);
        self.evaluated_paths
            .write()
            .unwrap()
            .replace(path.to_written());
    }
    pub(crate) fn increment_unchanged(&self) {
        self.unchanged.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn increment_matches(&self, num_matches: usize) {
        self.matches.fetch_add(num_matches, Ordering::Relaxed);
    }

    /// Send a message to the display thread
    pub(crate) fn push_message(&self, msg: impl Into<Message>) {
        self.messages.send(msg.into()).ok();
    }
}

impl TraversalContext for TraversalOptions<'_, '_> {
    fn interner(&self) -> &PathInterner {
        &self.interner
    }

    fn evaluated_paths(&self) -> BTreeSet<PgLSPath> {
        self.evaluated_paths.read().unwrap().clone()
    }

    fn push_diagnostic(&self, error: Error) {
        self.push_message(error);
    }

    fn can_handle(&self, pgls_path: &PgLSPath) -> bool {
        let path = pgls_path.as_path();

        let is_valid_file = self.fs.path_is_file(path)
            && path
                .extension()
                .is_some_and(|ext| ext == "sql" || ext == "pg");

        if self.fs.path_is_dir(path) || self.fs.path_is_symlink(path) || is_valid_file {
            // handle:
            // - directories
            // - symlinks
            // - unresolved symlinks
            //   e.g `symlink/subdir` where symlink points to a directory that includes `subdir`.
            //   Note that `symlink/subdir` is not an existing file.
            let can_handle = !self
                .workspace
                .is_path_ignored(IsPathIgnoredParams {
                    pgls_path: pgls_path.clone(),
                })
                .unwrap_or_else(|err| {
                    self.push_diagnostic(err.into());
                    false
                });
            return can_handle;
        }

        // bail on fifo and socket files
        if !is_valid_file {
            return false;
        }

        true
    }

    fn handle_path(&self, path: PgLSPath) {
        handle_file(self, &path)
    }

    fn store_path(&self, path: PgLSPath) {
        self.evaluated_paths
            .write()
            .unwrap()
            .insert(PgLSPath::new(path.as_path()));
    }
}

/// This function wraps the [process_file] function implementing the traversal
/// in a [catch_unwind] block and emit diagnostics in case of error (either the
/// traversal function returns Err or panics)
fn handle_file(ctx: &TraversalOptions, path: &PgLSPath) {
    match catch_unwind(move || process_file(ctx, path)) {
        Ok(Ok(FileStatus::Changed)) => {
            ctx.increment_changed(path);
        }
        Ok(Ok(FileStatus::Unchanged)) => {
            ctx.increment_unchanged();
        }
        Ok(Ok(FileStatus::SearchResult(num_matches, msg))) => {
            ctx.increment_unchanged();
            ctx.increment_matches(num_matches);
            ctx.push_message(msg);
        }
        Ok(Ok(FileStatus::Message(msg))) => {
            ctx.increment_unchanged();
            ctx.push_message(msg);
        }
        Ok(Ok(FileStatus::Protected(file_path))) => {
            ctx.increment_unchanged();
            ctx.push_diagnostic(WorkspaceError::protected_file(file_path).into());
        }
        Ok(Ok(FileStatus::Ignored)) => {}
        Ok(Err(err)) => {
            ctx.increment_unchanged();
            ctx.skipped.fetch_add(1, Ordering::Relaxed);
            ctx.push_message(err);
        }
        Err(err) => {
            let message = match err.downcast::<String>() {
                Ok(msg) => format!("processing panicked: {msg}"),
                Err(err) => match err.downcast::<&'static str>() {
                    Ok(msg) => format!("processing panicked: {msg}"),
                    Err(_) => String::from("processing panicked"),
                },
            };

            ctx.push_message(
                PanicDiagnostic { message }.with_file_path(path.display().to_string()),
            );
        }
    }
}
