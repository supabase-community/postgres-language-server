//! Generated file, do not edit by hand, see `xtask/codegen`

pub mod correctness;
pub mod destructive;
pub mod safety;
pub mod style;
pub mod typecheck;
::pgls_analyse::declare_category! { pub Lint { kind : Lint , groups : [self :: correctness :: Correctness , self :: destructive :: Destructive , self :: safety :: Safety , self :: style :: Style , self :: typecheck :: Typecheck ,] } }
