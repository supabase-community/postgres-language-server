BEGIN;

SET LOCAL enable_seqscan = false;

SET LOCAL enable_indexonlyscan = false;

SET LOCAL enable_bitmapscan = false;

CREATE TABLE clean_aborted_self(key int, data text);

CREATE INDEX clean_aborted_self_key ON clean_aborted_self(key);

INSERT INTO clean_aborted_self (key, data) VALUES (-1, 'just to allocate metapage');

SELECT pg_relation_size('clean_aborted_self_key') AS clean_aborted_self_key_before ;

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

SELECT 'clean_aborted_self_key_before' AS size_before, pg_relation_size('clean_aborted_self_key') size_after
WHERE 'clean_aborted_self_key_before' != pg_relation_size('clean_aborted_self_key');

ROLLBACK;
