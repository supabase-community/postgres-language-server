SELECT getdatabaseencoding() <> 'UTF8' OR
       (SELECT count(*) FROM pg_collation WHERE collname IN ('de_DE', 'en_US', 'sv_SE', 'tr_TR') AND collencoding = pg_char_to_encoding('UTF8')) <> 4 OR
       version() !~ 'linux-gnu'
       AS skip_test ;
