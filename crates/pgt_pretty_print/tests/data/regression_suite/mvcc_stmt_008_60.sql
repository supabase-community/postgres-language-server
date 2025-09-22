DO $$
BEGIN
    -- iterate often enough to see index growth even on larger-than-default page sizes
    FOR i IN 1..100 LOOP
        BEGIN
	    -- perform index scan over all the inserted keys to get them to be seen as dead
            IF EXISTS(SELECT * FROM clean_aborted_self WHERE key > 0 AND key < 100) THEN
	        RAISE data_corrupted USING MESSAGE = 'these rows should not exist';
            END IF;
            INSERT INTO clean_aborted_self SELECT g.i, 'rolling back in a sec' FROM generate_series(1, 100) g(i);
	    -- just some error that's not normally thrown
	    RAISE reading_sql_data_not_permitted USING MESSAGE = 'round and round again';
	EXCEPTION WHEN reading_sql_data_not_permitted THEN END;
    END LOOP;
END;$$;
