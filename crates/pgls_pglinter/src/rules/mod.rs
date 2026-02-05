//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod base;
pub mod cluster;
pub mod schema;
::pgls_analyse::declare_category! { pub PgLinter { kind : Lint , groups : [self :: base :: Base , self :: cluster :: Cluster , self :: schema :: Schema ,] } }
