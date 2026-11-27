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

    /// Anchors the relative globs at `root`, see [Matcher::matches_path].
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
    /// It returns [true] if there's at least one match. With a root, a relative glob only matches
    /// the paths below the root, from the root: with the root `/repo`, `sql/*.sql` matches
    /// `/repo/sql/a.sql` but not `/repo/app/sql/a.sql`, and `**/test` matches every `test` folder
    /// below `/repo`. Without a root, a relative glob matches at any depth. An absolute glob
    /// matches the whole path.
    ///
    /// A pattern also matches the paths below the file or folder it names, since the pattern
    /// library doesn't support single files. Given the pattern `out`:
    /// - `out/index.html` -> matches
    /// - `out/` -> matches
    /// - `layout.tsx` -> does not match
    /// - `routes/foo.ts` -> does not match
    ///
    /// With the root `/repo`, `out` matches `/repo/out` and the paths below it, but not
    /// `/repo/a/out`.
    pub fn matches_path(&self, source: &Path) -> bool {
        let below_root = self
            .root
            .as_deref()
            .map(|root| source.strip_prefix(root).ok());
        self.patterns.iter().any(|pattern| match below_root {
            Some(relative) if pattern.is_relative() => relative.is_some_and(|relative| {
                let named = Path::new(pattern.as_str());
                pattern.matches_relative_path_with(relative, self.options)
                    || relative.starts_with(named.strip_prefix(".").unwrap_or(named))
            }),
            _ => {
                pattern.matches_path_with(source, self.options)
                    || source
                        .ancestors()
                        .any(|ancestor| ancestor.ends_with(pattern.as_str()))
            }
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

    impl Matcher {
        /// A matcher of `patterns`, anchored at `root` if there is one.
        fn of(root: Option<&str>, patterns: &[&str]) -> Self {
            let mut matcher = Matcher::new(MatchOptions::default());
            if let Some(root) = root {
                matcher.set_root(PathBuf::from(root));
            }
            for pattern in patterns {
                matcher.add_pattern(pattern).unwrap();
            }
            matcher
        }

        fn matches_str(&self, path: &str) -> bool {
            self.matches_path(Path::new(path))
        }
    }

    /// A relative glob matches from the root, not at any depth.
    #[test]
    fn relative_glob_matches_from_the_root() {
        let matcher = Matcher::of(Some("/repo"), &["sql/**/*.sql"]);
        assert!(matcher.matches_str("/repo/sql/a.sql"));
        assert!(matcher.matches_str("/repo/sql/b/a.sql"));
        assert!(!matcher.matches_str("/repo/app/sql/a.sql"));
    }

    /// `**/` matches at any depth below the root.
    #[test]
    fn leading_recursive_wildcard_matches_at_any_depth_below_the_root() {
        let matcher = Matcher::of(Some("/repo"), &["**/test"]);
        assert!(matcher.matches_str("/repo/test"));
        assert!(matcher.matches_str("/repo/a/b/test"));
        assert!(!matcher.matches_str("/other/test"));

        let matcher = Matcher::of(Some("/repo"), &["**/*.sql"]);
        assert!(matcher.matches_str("/repo/a.sql"));
        assert!(matcher.matches_str("/repo/a/b.sql"));
    }

    /// A name matches the file or folder of that name at the root and the paths below it.
    #[test]
    fn name_matches_below_the_root() {
        let matcher = Matcher::of(Some("/repo"), &["test"]);
        assert!(matcher.matches_str("/repo/test"));
        assert!(matcher.matches_str("/repo/test/a.sql"));
        assert!(!matcher.matches_str("/repo/a/test"));
        assert!(!matcher.matches_str("/repo/a/test/a.sql"));
        assert!(!matcher.matches_str("/repo/tests/a.sql"));

        let matcher = Matcher::of(Some("/repo"), &["a/test"]);
        assert!(matcher.matches_str("/repo/a/test/a.sql"));
        assert!(!matcher.matches_str("/repo/b/a/test/a.sql"));
    }

    /// `./` in front of a relative glob changes nothing.
    #[test]
    fn current_folder_prefix_is_the_same_as_none() {
        for pattern in ["./sql/*.sql", "sql/*.sql"] {
            let matcher = Matcher::of(Some("/repo"), &[pattern]);
            assert!(matcher.matches_str("/repo/sql/a.sql"), "{pattern}");
            assert!(!matcher.matches_str("/repo/app/sql/a.sql"), "{pattern}");
        }
        let matcher = Matcher::of(Some("/repo"), &["./test"]);
        assert!(matcher.matches_str("/repo/test/a.sql"));
        assert!(!matcher.matches_str("/repo/a/test/a.sql"));
    }

    /// An absolute glob matches the whole path, whatever the root.
    #[test]
    fn absolute_glob_matches_the_whole_path() {
        let matcher = Matcher::of(Some("/repo"), &["/other/**/*.sql"]);
        assert!(matcher.matches_str("/other/a/b.sql"));
        assert!(!matcher.matches_str("/repo/other/a/b.sql"));

        let matcher = Matcher::of(Some("/repo"), &["/repo/sql"]);
        assert!(matcher.matches_str("/repo/sql"));
        assert!(matcher.matches_str("/repo/sql/a.sql"));
    }

    /// A relative glob doesn't match a path outside the root.
    #[test]
    fn relative_glob_does_not_match_outside_the_root() {
        let matcher = Matcher::of(Some("/repo/db"), &["**/*.sql", "test"]);
        assert!(matcher.matches_str("/repo/db/a.sql"));
        assert!(!matcher.matches_str("/repo/a.sql"));
        assert!(!matcher.matches_str("/repo/test/a.sql"));
        assert!(!matcher.matches_str("db/a.sql"));
    }

    /// Without a root, a relative glob matches at any depth.
    #[test]
    fn relative_glob_without_a_root_matches_at_any_depth() {
        let matcher = Matcher::of(None, &["sql/*.sql", "test"]);
        assert!(matcher.matches_str("/repo/sql/a.sql"));
        assert!(matcher.matches_str("/repo/app/sql/a.sql"));
        assert!(matcher.matches_str("/repo/a/test/a.sql"));
    }

    /// Alternatives in braces match from the root too.
    #[test]
    fn alternatives_match_from_the_root() {
        let mut matcher = Matcher::of(Some("/repo"), &[]);
        matcher
            .patterns
            .push(crate::Pattern::parse("{sql,db}/*.sql", true).unwrap());
        assert!(matcher.matches_str("/repo/sql/a.sql"));
        assert!(matcher.matches_str("/repo/db/a.sql"));
        assert!(!matcher.matches_str("/repo/app/sql/a.sql"));
        assert!(!matcher.matches_str("/repo/other/a.sql"));
    }
}
