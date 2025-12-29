//! Generic SARIF (Static Analysis Results Interchange Format) parser
//!
//! SARIF is a standard format for static analysis tool output.
//! See: https://sarifweb.azurewebsites.net/

use serde::Deserialize;

/// SARIF 2.1.0 root object
#[derive(Debug, Deserialize)]
pub struct SarifLog {
    #[serde(default)]
    pub runs: Vec<Run>,
}

/// A single run of a static analysis tool
#[derive(Debug, Deserialize)]
pub struct Run {
    #[serde(default)]
    pub results: Vec<Result>,
    pub tool: Option<Tool>,
}

/// Information about the tool that produced the results
#[derive(Debug, Deserialize)]
pub struct Tool {
    pub driver: Option<Driver>,
}

/// The tool driver (main component)
#[derive(Debug, Deserialize)]
pub struct Driver {
    pub name: Option<String>,
    pub version: Option<String>,
}

/// A single result from the analysis
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    /// The rule ID that was violated
    pub rule_id: Option<String>,
    /// Severity level: "error", "warning", "note", "none"
    pub level: Option<String>,
    /// The result message
    pub message: Option<Message>,
    /// Locations where the issue was found
    #[serde(default)]
    pub locations: Vec<Location>,
}

/// A message with text content
#[derive(Debug, Deserialize)]
pub struct Message {
    pub text: Option<String>,
}

/// A location in the source
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub physical_location: Option<PhysicalLocation>,
    pub logical_locations: Option<Vec<LogicalLocation>>,
}

/// A physical location (file, line, column)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalLocation {
    pub artifact_location: Option<ArtifactLocation>,
    pub region: Option<Region>,
}

/// Location of an artifact (file)
#[derive(Debug, Deserialize)]
pub struct ArtifactLocation {
    pub uri: Option<String>,
}

/// A region within a file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub start_line: Option<u32>,
    pub start_column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// A logical location (schema, table, function name, etc.)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalLocation {
    pub name: Option<String>,
    pub fully_qualified_name: Option<String>,
    pub kind: Option<String>,
}

impl SarifLog {
    /// Parse SARIF JSON into a structured log
    pub fn parse(json: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get all results from all runs
    pub fn all_results(&self) -> impl Iterator<Item = &Result> {
        self.runs.iter().flat_map(|run| run.results.iter())
    }

    /// Check if there are any results
    pub fn has_results(&self) -> bool {
        self.runs.iter().any(|run| !run.results.is_empty())
    }
}

impl Result {
    /// Get the severity level, defaulting to "warning"
    pub fn level_str(&self) -> &str {
        self.level.as_deref().unwrap_or("warning")
    }

    /// Get the message text, defaulting to empty string
    pub fn message_text(&self) -> &str {
        self.message
            .as_ref()
            .and_then(|m| m.text.as_deref())
            .unwrap_or("")
    }

    /// Get logical location names (e.g., affected database objects)
    pub fn logical_location_names(&self) -> Vec<&str> {
        self.locations
            .iter()
            .filter_map(|loc| loc.logical_locations.as_ref())
            .flatten()
            .filter_map(|ll| ll.fully_qualified_name.as_deref().or(ll.name.as_deref()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_sarif() {
        let json = r#"{
            "runs": [{
                "results": [{
                    "ruleId": "B001",
                    "level": "warning",
                    "message": { "text": "Table without primary key" }
                }]
            }]
        }"#;

        let log = SarifLog::parse(json).unwrap();
        assert!(log.has_results());

        let results: Vec<_> = log.all_results().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id.as_deref(), Some("B001"));
        assert_eq!(results[0].level_str(), "warning");
        assert_eq!(results[0].message_text(), "Table without primary key");
    }

    #[test]
    fn test_parse_empty_sarif() {
        let json = r#"{"runs": [{"results": []}]}"#;
        let log = SarifLog::parse(json).unwrap();
        assert!(!log.has_results());
    }
}
