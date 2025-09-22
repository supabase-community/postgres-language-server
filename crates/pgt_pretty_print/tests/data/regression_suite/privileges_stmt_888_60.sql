DO $$BEGIN EXECUTE format(
	'ALTER DATABASE %I OWNER TO regress_priv_group2', current_catalog); END$$;
