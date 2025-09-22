SELECT makeaclitem('regress_priv_user1'::regrole, 'regress_priv_user2'::regrole,
	'SELECT, fake_privilege', FALSE);
