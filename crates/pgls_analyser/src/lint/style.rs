//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod ban_char_field;
pub mod creating_enum;
pub mod prefer_big_int;
pub mod prefer_identity;
pub mod prefer_jsonb;
pub mod prefer_text_field;
pub mod prefer_timestamptz;
declare_lint_group! { pub Style { name : "style" , rules : [self :: ban_char_field :: BanCharField , self :: creating_enum :: CreatingEnum , self :: prefer_big_int :: PreferBigInt , self :: prefer_identity :: PreferIdentity , self :: prefer_jsonb :: PreferJsonb , self :: prefer_text_field :: PreferTextField , self :: prefer_timestamptz :: PreferTimestamptz ,] } }
