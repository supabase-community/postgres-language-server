//! "Virtual nodes": constructs that several statements print the same way but that aren't a
//! parser node of their own, such as `IF NOT EXISTS`, a window frame or a FILTER clause.

pub(crate) mod modifiers;
pub(crate) mod object_type;
pub(crate) mod options;
pub(crate) mod window_frame;
