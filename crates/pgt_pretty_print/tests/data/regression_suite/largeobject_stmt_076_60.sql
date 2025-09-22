SELECT pageno, data FROM pg_largeobject WHERE loid = (SELECT loid from lotest_stash_values)
EXCEPT
SELECT pageno, data FROM pg_largeobject WHERE loid = 'newloid';
