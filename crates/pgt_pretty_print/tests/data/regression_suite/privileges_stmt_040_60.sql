SELECT grantor::regrole FROM pg_auth_members WHERE roleid = 'regress_priv_user1'::regrole and member = 'regress_priv_user4'::regrole;
