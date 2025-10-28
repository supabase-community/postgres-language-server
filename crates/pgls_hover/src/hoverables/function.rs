use std::fmt::Write;

use pgls_schema_cache::{Function, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

impl ToHoverMarkdown for Function {
    fn footer_markdown_type(&self) -> &'static str {
        "sql"
    }

    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}.{}", self.schema, self.name)?;

        if let Some(args) = &self.argument_types {
            write!(writer, "({args})")?;
        } else {
            write!(writer, "()")?;
        }

        write!(
            writer,
            " → {}`",
            self.return_type.as_ref().unwrap_or(&"void".to_string())
        )?;

        Ok(())
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        let kind_text = match self.kind {
            pgls_schema_cache::ProcKind::Function => "Function",
            pgls_schema_cache::ProcKind::Procedure => "Procedure",
            pgls_schema_cache::ProcKind::Aggregate => "Aggregate",
            pgls_schema_cache::ProcKind::Window => "Window",
        };

        write!(writer, "{kind_text}")?;

        let behavior_text = match self.behavior {
            pgls_schema_cache::Behavior::Immutable => " - Immutable",
            pgls_schema_cache::Behavior::Stable => " - Stable",
            pgls_schema_cache::Behavior::Volatile => "",
        };

        write!(writer, "{behavior_text}")?;

        if self.security_definer {
            write!(writer, " - Security DEFINER")?;
        } else {
            write!(writer, " - Security INVOKER")?;
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(def) = self.definition.as_ref() {
            /*
             * We don't want to show 250 lines of functions to the user.
             * If we have more than 30 lines, we'll only show the signature.
             */
            if def.lines().count() > 30 {
                let without_boilerplate: String = def
                    .split_ascii_whitespace()
                    .skip_while(|elem| {
                        ["create", "or", "replace", "function"]
                            .contains(&elem.to_ascii_lowercase().as_str())
                    })
                    .collect::<Vec<&str>>()
                    .join(" ");

                for char in without_boilerplate.chars() {
                    match char {
                        '(' => {
                            write!(writer, "(\n  ")?;
                        }

                        ')' => {
                            write!(writer, "\n)\n")?;
                            break;
                        }

                        ',' => {
                            // one space already present
                            write!(writer, ",\n ")?;
                        }

                        _ => {
                            write!(writer, "{char}")?;
                        }
                    }
                }
            } else {
                write!(writer, "\n{def}\n")?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ContextualPriority for Function {
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        // built-in functions get higher priority
        if self.language == "internal" {
            score += 100.0;
        }

        // public schema functions get base priority
        if self.schema == "public" {
            score += 50.0;
        } else {
            score += 25.0;
        }

        // aggregate and window functions are commonly used
        match self.kind {
            pgls_schema_cache::ProcKind::Aggregate => score += 20.0,
            pgls_schema_cache::ProcKind::Window => score += 15.0,
            pgls_schema_cache::ProcKind::Function => score += 10.0,
            pgls_schema_cache::ProcKind::Procedure => score += 5.0,
        }

        score
    }
}
