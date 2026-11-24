use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let grammar_file = manifest_dir.join("grammar.js");
    let config_file = manifest_dir.join("tree-sitter.json");
    let src_dir = manifest_dir.join("src");
    let scanner_path = src_dir.join("scanner.c");
    let generated_dir = out_dir.join("generated");
    let parser_path = generated_dir.join("parser.c");
    let node_types_path = generated_dir.join("node-types.json");
    let stamp_path = generated_dir.join(".stamp");

    println!("cargo:rerun-if-changed={}", grammar_file.display());
    println!("cargo:rerun-if-changed={}", config_file.display());
    println!("cargo:rerun-if-changed={}", scanner_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );

    // Detect Emscripten target for WASM builds.
    let target = std::env::var("TARGET").unwrap_or_default();
    let is_emscripten = target.contains("emscripten");

    std::fs::create_dir_all(&generated_dir)?;

    let stamp = compute_stamp([&grammar_file, &config_file]);
    let should_regenerate =
        !parser_path.exists() || !node_types_path.exists() || read_stamp(&stamp_path) != stamp;

    if should_regenerate {
        generate_grammar(&grammar_file, &config_file, &generated_dir);
        std::fs::write(&stamp_path, &stamp)?;
    }

    let parser_source = std::fs::read_to_string(&parser_path)?;
    ParserTables::read(&parser_source)?.write(&out_dir.join("kinds.rs"))?;

    let mut c_config = cc::Build::new();

    // Use Emscripten compiler for WASM builds.
    if is_emscripten {
        c_config.compiler("emcc").archiver("emar");
    }

    // Generated parser.c includes tree_sitter headers from generated_dir/tree_sitter.
    // scanner.c still lives in src/, so both include roots are required.
    c_config
        .std("c11")
        .include(&generated_dir)
        .include(&src_dir);

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    c_config.file(&parser_path);
    if scanner_path.exists() {
        c_config.file(&scanner_path);
    }

    c_config.compile("tree_sitter_pgls");
    Ok(())
}

fn compute_stamp(files: [&Path; 2]) -> String {
    let mut hasher = DefaultHasher::new();

    for file in files {
        file.as_os_str().hash(&mut hasher);
        let contents = std::fs::read(file).unwrap_or_else(|error| {
            panic!("Failed to read {}: {error}", file.display());
        });
        contents.hash(&mut hasher);
    }

    format!("{:016x}", hasher.finish())
}

fn read_stamp(stamp_path: &Path) -> String {
    std::fs::read_to_string(stamp_path)
        .map(|value| value.trim().to_owned())
        .unwrap_or_default()
}

/// How often the build script runs `tree-sitter generate` before it gives up. The generator needs
/// about 2.5 GB of memory and up to four minutes on a loaded machine. When it gets killed, e.g. by
/// the OOM killer while several worktrees build at once, it exits without any output, and running
/// it again is the fix.
const GENERATE_ATTEMPTS_MAX: u32 = 3;

fn generate_grammar(grammar_file: &Path, config_file: &Path, generated_dir: &Path) {
    for attempt in 1..=GENERATE_ATTEMPTS_MAX {
        let failure = match run_generate(grammar_file, config_file, generated_dir) {
            Ok(()) => return,
            Err(failure) => failure,
        };

        if failure.is_transient() && attempt < GENERATE_ATTEMPTS_MAX {
            println!(
                "cargo:warning=tree-sitter generate failed without an error ({}), attempt {attempt} of {GENERATE_ATTEMPTS_MAX}; trying again",
                failure.status
            );
            continue;
        }

        panic!(
            "Failed to generate the tree-sitter grammar (attempt {attempt} of {GENERATE_ATTEMPTS_MAX}).\n\
             command: tree-sitter generate grammar.js --output {}\n\
             {}{}\nstdout:\n{}\nstderr:\n{}",
            generated_dir.display(),
            failure.status,
            if failure.status.code().is_none() {
                " (killed by a signal; tree-sitter generate needs about 2.5 GB of memory, so check for the OOM killer)"
            } else {
                ""
            },
            String::from_utf8_lossy(&failure.stdout),
            String::from_utf8_lossy(&failure.stderr),
        );
    }
}

fn run_generate(
    grammar_file: &Path,
    config_file: &Path,
    generated_dir: &Path,
) -> Result<(), GenerateFailure> {
    // tree-sitter generate updates tree-sitter.json in its working directory.
    // Use an isolated temp workdir under OUT_DIR to avoid mutating repository files.
    let generator_workdir = generated_dir.join("tree-sitter-workdir");
    let work_grammar = generator_workdir.join("grammar.js");
    let work_config = generator_workdir.join("tree-sitter.json");

    let _ = std::fs::remove_dir_all(&generator_workdir);
    std::fs::create_dir_all(&generator_workdir).unwrap_or_else(|error| {
        panic!("Failed to create the tree-sitter generator workdir: {error}");
    });
    std::fs::copy(grammar_file, &work_grammar).unwrap_or_else(|error| {
        panic!(
            "Failed to copy {} into generator workdir: {error}",
            grammar_file.display()
        );
    });
    std::fs::copy(config_file, &work_config).unwrap_or_else(|error| {
        panic!(
            "Failed to copy {} into generator workdir: {error}",
            config_file.display()
        );
    });

    let output = Command::new("tree-sitter")
        .arg("generate")
        .arg("grammar.js")
        .arg("--output")
        .arg(generated_dir)
        .current_dir(&generator_workdir)
        .output();

    let _ = std::fs::remove_dir_all(&generator_workdir);

    match output {
        Ok(result) if result.status.success() => Ok(()),
        Ok(result) => Err(GenerateFailure {
            status: result.status,
            stdout: result.stdout,
            stderr: result.stderr,
        }),
        Err(error) => {
            panic!(
                "tree-sitter CLI not found ({error}). Please install it with: `just install-tools`"
            );
        }
    }
}

struct GenerateFailure {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl GenerateFailure {
    /// A generator that got killed, or failed without saying why. A real error in the grammar
    /// exits with a code and a message on stderr.
    fn is_transient(&self) -> bool {
        self.status.code().is_none()
            || (self.stdout.trim_ascii().is_empty() && self.stderr.trim_ascii().is_empty())
    }
}

/// The symbol and field tables of the generated `parser.c`, from which `kinds.rs` is written.
struct ParserTables<'a> {
    /// The kinds, in the order of `parser.c`: `(C identifier of the public symbol, public id,
    /// name, named)`, one per name and namedness of a visible symbol, with the id that
    /// [`ts_language_symbol_for_name`] returns.
    ///
    /// [`ts_language_symbol_for_name`]: https://github.com/tree-sitter/tree-sitter/blob/v0.25.10/lib/src/language.c#L183
    kinds: Vec<(&'a str, u16, &'a str, bool)>,
    /// The fields: `(name, id)`, in id order.
    fields: Vec<(&'a str, u16)>,
}

impl<'a> ParserTables<'a> {
    fn read(source: &'a str) -> Result<Self, String> {
        let ids: HashMap<&str, u16> = Self::block(source, "enum ts_symbol_identifiers {")?
            .lines()
            .filter_map(|line| line.trim().trim_end_matches(',').split_once(" = "))
            .map(|(ident, id)| Ok((ident, id.parse().map_err(|e| format!("{ident}: {e}"))?)))
            .collect::<Result<_, String>>()?;
        let id_of = |ident: &str| -> Result<u16, String> {
            ids.get(ident)
                .copied()
                .ok_or_else(|| format!("no id for symbol `{ident}`"))
        };

        let names: HashMap<&str, &str> = Self::entries(source, "ts_symbol_names[] = {")?
            .map(|(ident, value)| Ok((ident, Self::string_literal(value)?)))
            .collect::<Result<_, String>>()?;
        let public: HashMap<&str, &str> = Self::entries(source, "ts_symbol_map[] = {")?.collect();

        let mut kinds = Vec::new();
        let mut seen = HashMap::new();
        let metadata = Self::block(source, "ts_symbol_metadata[] = {")?;
        let mut current: Option<(&str, bool, bool)> = None;
        for line in metadata.lines().map(str::trim) {
            if let Some(ident) = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix("] = {"))
            {
                current = Some((ident, false, false));
                continue;
            }
            match (line, current.as_mut()) {
                (".visible = true,", Some((_, visible, _))) => *visible = true,
                (".named = true,", Some((_, _, named))) => *named = true,
                ("},", Some(_)) => {
                    let Some((ident, true, named)) = current.take() else {
                        continue;
                    };
                    let name = *names
                        .get(ident)
                        .ok_or_else(|| format!("no name for symbol `{ident}`"))?;
                    let public_ident = *public
                        .get(ident)
                        .ok_or_else(|| format!("no public symbol for `{ident}`"))?;
                    // `Node::kind_id` returns the public symbol, so a kind is one id only if all
                    // its symbols share their public symbol, which tree-sitter's generator does.
                    let id = id_of(public_ident)?;
                    match seen.insert((name, named), id) {
                        None => kinds.push((public_ident, id, name, named)),
                        Some(first) if first == id => {}
                        Some(first) => {
                            return Err(format!(
                                "kind `{name}` has the public symbols {first} and {id}"
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        let field_ids: HashMap<&str, u16> = Self::block(source, "enum ts_field_identifiers {")?
            .lines()
            .filter_map(|line| line.trim().trim_end_matches(',').split_once(" = "))
            .map(|(ident, id)| Ok((ident, id.parse().map_err(|e| format!("{ident}: {e}"))?)))
            .collect::<Result<_, String>>()?;
        let mut fields = Self::entries(source, "ts_field_names[] = {")?
            .filter(|(ident, _)| *ident != "0")
            .map(|(ident, value)| {
                let id = field_ids
                    .get(ident)
                    .copied()
                    .ok_or_else(|| format!("no id for field `{ident}`"))?;
                Ok((Self::string_literal(value)?, id))
            })
            .collect::<Result<Vec<_>, String>>()?;
        fields.sort_by_key(|(_, id)| *id);

        Ok(Self { kinds, fields })
    }

    /// The lines between `header` and the `};` that closes it.
    fn block(source: &'a str, header: &str) -> Result<&'a str, String> {
        let start = source
            .find(header)
            .ok_or_else(|| format!("`{header}` not found in parser.c"))?
            + header.len();
        source
            .get(start..)
            .and_then(|rest| rest.get(..rest.find("\n};")?))
            .ok_or_else(|| format!("`{header}` isn't closed in parser.c"))
    }

    /// The `[key] = value,` lines of the table after `header`.
    fn entries(
        source: &'a str,
        header: &str,
    ) -> Result<impl Iterator<Item = (&'a str, &'a str)>, String> {
        Ok(Self::block(source, header)?.lines().filter_map(|line| {
            let (key, value) = line.trim().split_once(" = ")?;
            let key = key.strip_prefix('[')?.strip_suffix(']')?;
            Some((key, value.trim_end_matches(',')))
        }))
    }

    /// The text of a C string literal. The grammar's names need no escapes; one with an escape is
    /// rejected rather than copied wrongly.
    fn string_literal(value: &'a str) -> Result<&'a str, String> {
        value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .filter(|v| !v.contains('\\'))
            .ok_or_else(|| format!("unexpected name literal {value}"))
    }

    /// The Rust constant of a kind: the name in upper case for named kinds, and the C identifier
    /// without its `anon_sym_` prefix after `ANON_` for anonymous ones, since `(` isn't an
    /// identifier.
    fn kind_constant(ident: &str, name: &str, named: bool) -> String {
        if named {
            name.to_ascii_uppercase()
        } else {
            let stem = ident
                .strip_prefix("anon_sym_")
                .or_else(|| ident.strip_prefix("anon_alias_sym_"))
                .unwrap_or(ident);
            format!("ANON_{}", stem.to_ascii_uppercase())
        }
    }

    fn write(&self, out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut out = String::from("// Generated by build.rs from parser.c. Don't edit.\n\n");
        let mut constants = HashSet::new();

        out.push_str("impl Kind {\n");
        for (ident, id, name, named) in &self.kinds {
            let constant = Self::kind_constant(ident, name, *named);
            if !Self::is_identifier(&constant) || !constants.insert(constant.clone()) {
                return Err(format!("kind `{name}` gives the constant `{constant}`, which is taken or not an identifier").into());
            }
            writeln!(
                out,
                "    /// `{name}`\n    pub const {constant}: Kind = Kind({id});"
            )?;
        }
        out.push_str("}\n\n");

        let names_len = self
            .kinds
            .iter()
            .map(|(_, id, ..)| *id as usize + 1)
            .max()
            .unwrap_or(0);
        let mut names = vec![""; names_len];
        for (_, id, name, _) in &self.kinds {
            if let Some(slot) = names.get_mut(*id as usize) {
                *slot = name;
            }
        }
        writeln!(
            out,
            "/// The kinds' names by id, `\"\"` for an id that isn't a kind's."
        )?;
        writeln!(out, "static KIND_NAMES: [&str; {names_len}] = {names:?};\n")?;

        constants.clear();
        out.push_str("impl Field {\n");
        for (name, id) in &self.fields {
            let constant = name.to_ascii_uppercase();
            if !Self::is_identifier(&constant) || !constants.insert(constant.clone()) {
                return Err(format!("field `{name}` gives the constant `{constant}`, which is taken or not an identifier").into());
            }
            writeln!(
                out,
                "    /// `{name}`\n    pub const {constant}: Field = Field({id});"
            )?;
        }
        out.push_str("}\n\n");

        let names_len = self
            .fields
            .iter()
            .map(|(_, id)| *id as usize + 1)
            .max()
            .unwrap_or(1);
        let mut names = vec![""; names_len];
        for (name, id) in &self.fields {
            if let Some(slot) = names.get_mut(*id as usize) {
                *slot = name;
            }
        }
        writeln!(
            out,
            "/// The fields' names by id, `\"\"` for an id that isn't a field's."
        )?;
        writeln!(out, "static FIELD_NAMES: [&str; {names_len}] = {names:?};")?;

        std::fs::write(out_path, out)?;
        Ok(())
    }

    fn is_identifier(constant: &str) -> bool {
        constant
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && constant
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}
