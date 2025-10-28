use std::fmt::Write;

use pgls_schema_cache::{Role, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

impl ToHoverMarkdown for pgls_schema_cache::Role {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}`", self.name)?;

        Ok(())
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(comm) = self.comment.as_ref() {
            write!(writer, "Comment: '{comm}'")?;
            writeln!(writer)?;
            writeln!(writer)?;
        }

        let mut permissions: Vec<&'static str> = vec![];

        if self.is_super_user {
            permissions.push("🔐 is superuser");
        }

        if self.can_login {
            permissions.push("🔑 can login");
        }

        if self.can_create_db {
            permissions.push("🏗 can create DB");
        }

        if self.can_create_roles {
            permissions.push("👥 can create roles");
        }

        if self.can_bypass_rls {
            permissions.push("🛡 can bypass RLS");
        }

        if !permissions.is_empty() {
            write!(writer, "Permissions:  ")?;
            writeln!(writer)?;

            for perm in permissions {
                write!(writer, "- {perm}")?;
                writeln!(writer)?;
            }
            writeln!(writer)?;
        } else {
            write!(writer, "No extra permissions.")?;
            writeln!(writer)?;
            writeln!(writer)?;
        }

        if !self.member_of.is_empty() {
            write!(writer, "Member Of:")?;
            writeln!(writer)?;

            for mem in &self.member_of {
                write!(writer, "- {mem}")?;
                writeln!(writer)?;
            }

            writeln!(writer)?;
        }

        if !self.has_member.is_empty() {
            write!(writer, "Has Members:")?;
            writeln!(writer)?;

            for mem in &self.has_member {
                write!(writer, "- {mem}")?;
                writeln!(writer)?;
            }
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        _writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        Ok(false)
    }
}

impl ContextualPriority for Role {
    // there are no roles with duplicate names.
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        0.0
    }
}
