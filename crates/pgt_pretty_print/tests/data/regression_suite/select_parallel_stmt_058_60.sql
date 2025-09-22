select * from tenk1 where (unique1 + random())::integer not in
	(select ten from tenk2);
