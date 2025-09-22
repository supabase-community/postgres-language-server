SELECT
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'USAGE') as priv,
	pg_has_role('regress_priv_user1', 'pg_database_owner', 'MEMBER') as mem,
	pg_has_role('regress_priv_user1', 'pg_database_owner',
				'MEMBER WITH ADMIN OPTION') as admin;
