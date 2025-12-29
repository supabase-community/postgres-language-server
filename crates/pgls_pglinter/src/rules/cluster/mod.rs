//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod password_encryption_is_md5;
pub mod pg_hba_entries_with_method_trust_or_password_should_not_exists;
pub mod pg_hba_entries_with_method_trust_should_not_exists;
::pgls_analyse::declare_lint_group! { pub Cluster { name : "cluster" , rules : [self :: password_encryption_is_md5 :: PasswordEncryptionIsMd5 , self :: pg_hba_entries_with_method_trust_or_password_should_not_exists :: PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists , self :: pg_hba_entries_with_method_trust_should_not_exists :: PgHbaEntriesWithMethodTrustShouldNotExists ,] } }
