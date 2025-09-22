INSERT INTO testpub_insert_onconfl_no_ri VALUES (1, 1) ON CONFLICT (a) DO UPDATE SET b = 2;
