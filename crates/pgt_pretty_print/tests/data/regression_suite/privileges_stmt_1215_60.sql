SELECT member::regrole::text, CASE WHEN grantor = 10 THEN 'BOOTSTRAP SUPERUSER' ELSE grantor::regrole::text END FROM pg_auth_members WHERE roleid = 'regress_group'::regrole ORDER BY 1, 2;
