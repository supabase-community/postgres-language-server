SET bytea_output TO escape;

CREATE ROLE regress_lo_user;

SELECT lo_create(42);

ALTER LARGE OBJECT 42 OWNER TO regress_lo_user;

SET SESSION AUTHORIZATION regress_lo_user;

GRANT SELECT ON LARGE OBJECT 42 TO public;

COMMENT ON LARGE OBJECT 42 IS 'the ultimate answer';

RESET SESSION AUTHORIZATION;

CREATE TABLE lotest_stash_values (loid oid, fd integer);

INSERT INTO lotest_stash_values (loid) SELECT lo_creat(42);

BEGIN;

UPDATE lotest_stash_values SET fd = lo_open(loid, CAST(x'20000' | x'40000' AS integer));

SELECT lowrite(fd, '
I wandered lonely as a cloud
That floats on high o''er vales and hills,
When all at once I saw a crowd,
A host, of golden daffodils;
Beside the lake, beneath the trees,
Fluttering and dancing in the breeze.

Continuous as the stars that shine
And twinkle on the milky way,
They stretched in never-ending line
Along the margin of a bay:
Ten thousand saw I at a glance,
Tossing their heads in sprightly dance.

The waves beside them danced; but they
Out-did the sparkling waves in glee:
A poet could not but be gay,
In such a jocund company:
I gazed--and gazed--but little thought
What wealth the show to me had brought:

For oft, when on my couch I lie
In vacant or in pensive mood,
They flash upon that inward eye
Which is the bliss of solitude;
And then my heart with pleasure fills,
And dances with the daffodils.

         -- William Wordsworth
') FROM lotest_stash_values;

SELECT lo_close(fd) FROM lotest_stash_values;

END;

SELECT lo_from_bytea(0, lo_get(loid)) AS newloid FROM lotest_stash_values

BEGIN;

UPDATE lotest_stash_values SET fd=lo_open(loid, CAST(x'20000' | x'40000' AS integer));

SELECT lo_lseek(fd, 104, 0) FROM lotest_stash_values;

SELECT loread(fd, 28) FROM lotest_stash_values;

SELECT lo_lseek(fd, -19, 1) FROM lotest_stash_values;

SELECT lowrite(fd, 'n') FROM lotest_stash_values;

SELECT lo_tell(fd) FROM lotest_stash_values;

SELECT lo_lseek(fd, -744, 2) FROM lotest_stash_values;

SELECT loread(fd, 28) FROM lotest_stash_values;

SELECT lo_close(fd) FROM lotest_stash_values;

END;

BEGIN;

SELECT lo_open(loid, x'40000'::int) from lotest_stash_values;

ABORT;

DO 'dobody';

BEGIN;

UPDATE lotest_stash_values SET fd=lo_open(loid, CAST(x'20000' | x'40000' AS integer));

SELECT lo_truncate(fd, 11) FROM lotest_stash_values;

SELECT loread(fd, 15) FROM lotest_stash_values;

SELECT lo_truncate(fd, 10000) FROM lotest_stash_values;

SELECT loread(fd, 10) FROM lotest_stash_values;

SELECT lo_lseek(fd, 0, 2) FROM lotest_stash_values;

SELECT lo_tell(fd) FROM lotest_stash_values;

SELECT lo_truncate(fd, 5000) FROM lotest_stash_values;

SELECT lo_lseek(fd, 0, 2) FROM lotest_stash_values;

SELECT lo_tell(fd) FROM lotest_stash_values;

SELECT lo_close(fd) FROM lotest_stash_values;

END;

BEGIN;

UPDATE lotest_stash_values SET fd = lo_open(loid, CAST(x'20000' | x'40000' AS integer));

SELECT lo_lseek64(fd, 4294967296, 0) FROM lotest_stash_values;

SELECT lowrite(fd, 'offset:4GB') FROM lotest_stash_values;

SELECT lo_tell64(fd) FROM lotest_stash_values;

SELECT lo_lseek64(fd, -10, 1) FROM lotest_stash_values;

SELECT lo_tell64(fd) FROM lotest_stash_values;

SELECT loread(fd, 10) FROM lotest_stash_values;

SELECT lo_truncate64(fd, 5000000000) FROM lotest_stash_values;

SELECT lo_lseek64(fd, 0, 2) FROM lotest_stash_values;

SELECT lo_tell64(fd) FROM lotest_stash_values;

SELECT lo_truncate64(fd, 3000000000) FROM lotest_stash_values;

SELECT lo_lseek64(fd, 0, 2) FROM lotest_stash_values;

SELECT lo_tell64(fd) FROM lotest_stash_values;

SELECT lo_close(fd) FROM lotest_stash_values;

END;

SELECT lo_unlink(loid) from lotest_stash_values;

TRUNCATE lotest_stash_values;

INSERT INTO lotest_stash_values (loid) SELECT lo_import('filename');

BEGIN;

UPDATE lotest_stash_values SET fd=lo_open(loid, CAST(x'20000' | x'40000' AS integer));

SELECT lo_lseek(fd, 0, 2) FROM lotest_stash_values;

SELECT lo_lseek(fd, 2030, 0) FROM lotest_stash_values;

SELECT loread(fd, 36) FROM lotest_stash_values;

SELECT lo_tell(fd) FROM lotest_stash_values;

SELECT lo_lseek(fd, -26, 1) FROM lotest_stash_values;

SELECT lowrite(fd, 'abcdefghijklmnop') FROM lotest_stash_values;

SELECT lo_lseek(fd, 2030, 0) FROM lotest_stash_values;

SELECT loread(fd, 36) FROM lotest_stash_values;

SELECT lo_close(fd) FROM lotest_stash_values;

END;

SELECT lo_export(loid, 'filename') FROM lotest_stash_values;

SELECT pageno, data FROM pg_largeobject WHERE loid = (SELECT loid from lotest_stash_values)
EXCEPT
SELECT pageno, data FROM pg_largeobject WHERE loid = 'newloid';

SELECT lo_unlink(loid) FROM lotest_stash_values;

TRUNCATE lotest_stash_values;

SELECT lo_from_bytea(0, lo_get('newloid_1')) AS newloid_2

SELECT fipshash(lo_get('newloid_1')) = fipshash(lo_get('newloid_2'));

SELECT lo_get('newloid_1', 0, 20);

SELECT lo_get('newloid_1', 10, 20);

SELECT lo_put('newloid_1', 5, decode('afafafaf', 'hex'));

SELECT lo_get('newloid_1', 0, 20);

SELECT lo_put('newloid_1', 4294967310, 'foo');

SELECT lo_get('newloid_1');

SELECT lo_get('newloid_1', 4294967294, 100);

SELECT lo_from_bytea(0, E'\\xdeadbeef') AS newloid

SET bytea_output TO hex;

SELECT lo_get('newloid');

SELECT lo_create(2121);

COMMENT ON LARGE OBJECT 2121 IS 'testing comments';

START TRANSACTION READ ONLY;

SELECT lo_open(2121, x'40000'::int);

SELECT lo_open(2121, x'20000'::int);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_create(42);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_creat(42);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_unlink(42);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lowrite(42, 'x');

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_import('filename');

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_truncate(42, 0);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_truncate64(42, 0);

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_from_bytea(0, 'x');

ROLLBACK;

START TRANSACTION READ ONLY;

SELECT lo_put(42, 0, 'x');

ROLLBACK;

DROP TABLE lotest_stash_values;

DROP ROLE regress_lo_user;
