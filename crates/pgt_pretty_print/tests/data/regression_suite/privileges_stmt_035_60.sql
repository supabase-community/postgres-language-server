SELECT member::regrole, admin_option FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole;
