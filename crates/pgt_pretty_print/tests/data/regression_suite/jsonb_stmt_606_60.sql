SELECT count(*) FROM (SELECT (jsonb_each(j)).key FROM testjsonb) AS wow;
