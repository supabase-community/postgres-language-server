//! Generated file, do not edit by hand, see `xtask/codegen`

pub(crate) mod correctness;
pub(crate) mod destructive;
pub(crate) mod safety;
pub(crate) mod style;
pub(crate) mod typecheck;
::pgls_analyse::declare_category! { pub Lint { kind : Lint , groups : [self :: correctness :: Correctness , self :: destructive :: Destructive , self :: safety :: Safety , self :: style :: Style , self :: typecheck :: Typecheck ,] } }
