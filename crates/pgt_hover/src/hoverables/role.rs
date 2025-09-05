use std::fmt::Write;

use pgt_schema_cache::Role;
use pgt_treesitter::TreesitterContext;

use crate::{
    contextual_priority::ContextualPriority,
    to_markdown::{ToHoverMarkdown, markdown_newline},
};

impl ToHoverMarkdown for pgt_schema_cache::Role {
    fn hover_headline<W: Write>(&self, writer: &mut W) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}`", self.name)?;

        Ok(())
    }

    fn hover_body<W: Write>(&self, writer: &mut W) -> Result<bool, std::fmt::Error> {
        if let Some(comm) = self.comment.as_ref() {
            write!(writer, "{}", comm)?;
            markdown_newline(writer)?;
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

        if self.can_bypass_rls {
            permissions.push("🛡 can bypass RLS");
        }

        if permissions.len() > 0 {
            write!(writer, "Permissions:  ")?;
            markdown_newline(writer)?;

            for perm in permissions {
                write!(writer, "- {}", perm)?;
                markdown_newline(writer)?;
            }
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(&self, _writer: &mut W) -> Result<bool, std::fmt::Error> {
        Ok(false)
    }
}

impl ContextualPriority for Role {
    // there are no roles with duplicate names.
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        0.0
    }
}
