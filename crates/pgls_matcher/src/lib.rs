pub mod pattern;

pub use pattern::{MatchOptions, Pattern, PatternError};
use pgls_console::markup;
use pgls_diagnostics::Diagnostic;
use std::path::{Path, PathBuf};

/// A data structure to use when there's need to match a string or a path a against
/// a unix shell style patterns
#[derive(Debug, Default)]
pub struct Matcher {
    root: Option<PathBuf>,
    patterns: Vec<Pattern>,
    options: MatchOptions,
}

impl Matcher {
    /// Creates a new Matcher with given options.
    ///
    /// Check [glob website](https://docs.rs/glob/latest/glob/struct.MatchOptions.html) for [MatchOptions]
    pub fn new(options: MatchOptions) -> Self {
        Self {
            root: None,
            patterns: Vec::new(),
            options,
        }
    }

    pub fn empty() -> Self {
        Self {
            root: None,
            patterns: Vec::new(),
            options: MatchOptions::default(),
        }
    }

    /// Sets the folder that relative patterns are relative to, the folder of the configuration
    /// file, see [`Self::matches_path`].
    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }

    /// It adds a unix shell style pattern
    pub fn add_pattern(&mut self, pattern: &str) -> Result<(), PatternError> {
        let pattern = Pattern::new(pattern)?;
        self.patterns.push(pattern);
        Ok(())
    }

    /// It matches the given string against the stored patterns.
    ///
    /// It returns [true] if at least one pattern matches the whole string: `public.users` doesn't
    /// match `public.users_archive`.
    pub fn matches(&self, source: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.matches_with(source, self.options))
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// It matches the given path against the stored patterns
    ///
    /// It returns [true] if there's at least one match. A path below the root also matches the
    /// patterns as a path relative to it: `migrations/*.sql` matches `<root>/migrations/a.sql`.
    pub fn matches_path(&self, source: &Path) -> bool {
        let relative = self
            .root
            .as_deref()
            .and_then(|root| source.strip_prefix(root).ok());
        self.patterns.iter().any(|pattern| {
            // Here we cover cases where the user specifies single files inside the patterns.
            // The pattern library doesn't support single files, we here we just do a check
            // on contains
            //
            // Given the pattern `out`:
            // - `out/index.html` -> matches
            // - `out/` -> matches
            // - `layout.tsx` -> does not match
            // - `routes/foo.ts` -> does not match
            pattern.matches_path_with(source, self.options)
                || relative
                    .is_some_and(|relative| pattern.matches_path_with(relative, self.options))
                || source
                    .ancestors()
                    .any(|ancestor| ancestor.ends_with(pattern.as_str()))
        })
    }
}

impl Diagnostic for PatternError {
    fn description(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.msg)
    }

    fn message(&self, fmt: &mut pgls_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        fmt.write_markup(markup!({ self.msg }))
    }
}

#[cfg(test)]
mod test {
    use crate::Matcher;
    use crate::pattern::MatchOptions;
    use std::env;
    use std::path::{Path, PathBuf};

    #[test]
    fn matches() {
        let current = env::current_dir().unwrap();
        let dir = format!("{}/**/*.rs", current.display());
        let mut ignore = Matcher::new(MatchOptions::default());
        ignore.add_pattern(&dir).unwrap();
        let path = env::current_dir().unwrap().join("src/lib.rs");
        let result = ignore.matches(path.to_str().unwrap());

        assert!(result);
    }

    /// A pattern matches the whole string, not a part of it.
    #[test]
    fn matches_the_whole_string() {
        let mut ignore = Matcher::new(MatchOptions::default());
        ignore.add_pattern("public.my_table").unwrap();
        ignore.add_pattern("*.temp_*").unwrap();

        assert!(ignore.matches("public.my_table"));
        assert!(!ignore.matches("public.my_table_archive"));
        assert!(!ignore.matches("xpublic.my_table"));
        assert!(ignore.matches("public.temp_users"));
        assert!(!ignore.matches("temp_users"));
    }

    #[test]
    fn matches_path() {
        let current = env::current_dir().unwrap();
        let dir = format!("{}/**/*.rs", current.display());
        let mut ignore = Matcher::new(MatchOptions::default());
        ignore.add_pattern(&dir).unwrap();
        let path = env::current_dir().unwrap().join("src/lib.rs");
        let result = ignore.matches_path(path.as_path());

        assert!(result);
    }

    #[test]
    fn matches_path_for_single_file_or_directory_name() {
        let dir = "inv";
        let valid_test_dir = "valid/";
        let mut ignore = Matcher::new(MatchOptions::default());
        ignore.add_pattern(dir).unwrap();
        ignore.add_pattern(valid_test_dir).unwrap();

        let path = env::current_dir().unwrap().join("tests").join("invalid");
        let result = ignore.matches_path(path.as_path());

        assert!(!result);

        let path = env::current_dir().unwrap().join("tests").join("valid");
        let result = ignore.matches_path(path.as_path());

        assert!(result);
    }

    #[test]
    fn matches_single_path() {
        let dir = "lib.rs";
        let mut ignore = Matcher::new(MatchOptions {
            require_literal_separator: true,
            case_sensitive: true,
            require_literal_leading_dot: true,
        });
        ignore.add_pattern(dir).unwrap();
        let path = env::current_dir().unwrap().join("src/lib.rs");
        // A file name matches the paths that end with it.
        assert!(ignore.matches_path(&path));
    }

    /// A relative pattern matches the paths below the root relative to it, not anywhere below it.
    #[test]
    fn matches_path_relative_to_the_root() {
        let mut ignore = Matcher::new(MatchOptions::default());
        ignore.set_root(PathBuf::from("/repo"));
        ignore.add_pattern("migrations/*.sql").unwrap();

        assert!(ignore.matches_path(Path::new("/repo/migrations/a.sql")));
        assert!(ignore.matches_path(Path::new("migrations/a.sql")));
        assert!(!ignore.matches_path(Path::new("/repo/db/migrations/a.sql")));
        assert!(!ignore.matches_path(Path::new("/other/migrations/a.sql")));
    }
}
