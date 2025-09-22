SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;
