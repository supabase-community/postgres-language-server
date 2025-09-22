SELECT * FROM information_schema.table_privileges t
	WHERE grantor LIKE 'regress_grantor%' ORDER BY ROW(t.*);
