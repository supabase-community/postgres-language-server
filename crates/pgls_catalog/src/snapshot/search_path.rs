use globset::Glob;

use super::Snapshot;

impl Snapshot {
    /// Expands the configured search path patterns against the schemas of the database.
    ///
    /// Patterns can be schema names or globs (e.g. `app_*`). The order of the patterns is kept,
    /// and `public` is always included last if it is not matched already.
    pub fn expand_search_path(&self, patterns: &[String]) -> Vec<String> {
        let mut schemas: Vec<String> = Vec::new();
        for pattern in patterns {
            let Ok(glob) = Glob::new(pattern) else {
                continue;
            };
            let matcher = glob.compile_matcher();
            for schema in &self.schemas {
                if matcher.is_match(schema.name.as_str()) && !schemas.contains(&schema.name) {
                    schemas.push(schema.name.clone());
                }
            }
        }

        if !schemas.iter().any(|schema| schema == "public") {
            schemas.push("public".to_string());
        }

        schemas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_globs_in_order() {
        let snapshot = Snapshot {
            schemas: ["public", "app_a", "app_b", "private"]
                .into_iter()
                .map(|name| super::super::Schema {
                    name: name.into(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        assert_eq!(
            snapshot.expand_search_path(&["private".into(), "app_*".into()]),
            ["private", "app_a", "app_b", "public"]
        );
    }
}
