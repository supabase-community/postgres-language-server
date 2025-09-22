SELECT backend_type FROM pg_stat_activity
WHERE CASE WHEN COALESCE(usesysid, 10) = 10 THEN terminate_nothrow(pid) END;
