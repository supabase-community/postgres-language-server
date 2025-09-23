SELECT 'first line'
' - next line'
	' - third line'
	AS "Three lines to one";

SET standard_conforming_strings TO on;

SELECT U&'d\0061t\+000061' AS U&"d\0061t\+000061";

SELECT U&'d!0061t\+000061' UESCAPE '!' AS U&"d*0061t\+000061" UESCAPE '*';

SELECT U&'a\\b' AS "a\b";

SELECT U&' \' UESCAPE '!' AS "tricky";

SELECT 'tricky' AS U&"\" UESCAPE '!';

SELECT E'd\u0061t\U00000061' AS "data";

SELECT E'a\\b' AS "a\b";

SET standard_conforming_strings TO off;

SELECT U&'d\0061t\+000061' AS U&"d\0061t\+000061";

SELECT U&'d!0061t\+000061' UESCAPE '!' AS U&"d*0061t\+000061" UESCAPE '*';

SELECT U&' \' UESCAPE '!' AS "tricky";

SELECT 'tricky' AS U&"\" UESCAPE '!';

RESET standard_conforming_strings;

SET bytea_output TO hex;

SELECT E'\\xDeAdBeEf'::bytea;

SELECT E'\\x De Ad Be Ef '::bytea;

SELECT E'\\xDeAdBeE'::bytea;

SELECT E'\\xDeAdBeEx'::bytea;

SELECT E'\\xDe00BeEf'::bytea;

SELECT E'DeAdBeEf'::bytea;

SELECT E'De\\000dBeEf'::bytea;

SELECT E'De\123dBeEf'::bytea;

SELECT E'De\\123dBeEf'::bytea;

SELECT E'De\\678dBeEf'::bytea;

SELECT E'DeAd\\\\BeEf'::bytea;

SELECT reverse(''::bytea);

SELECT reverse('\xaa'::bytea);

SELECT reverse('\xabcd'::bytea);

SET bytea_output TO escape;

SELECT E'\\xDeAdBeEf'::bytea;

SELECT E'\\x De Ad Be Ef '::bytea;

SELECT E'\\xDe00BeEf'::bytea;

SELECT E'DeAdBeEf'::bytea;

SELECT E'De\\000dBeEf'::bytea;

SELECT E'De\\123dBeEf'::bytea;

SELECT E'DeAd\\\\BeEf'::bytea;

SELECT pg_input_is_valid(E'\\xDeAdBeE', 'bytea');

SELECT * FROM pg_input_error_info(E'\\xDeAdBeE', 'bytea');

SELECT * FROM pg_input_error_info(E'\\xDeAdBeEx', 'bytea');

SELECT * FROM pg_input_error_info(E'foo\\99bar', 'bytea');

SELECT CAST(f1 AS text) AS "text(char)" FROM CHAR_TBL;

SELECT CAST(f1 AS text) AS "text(varchar)" FROM VARCHAR_TBL;

SELECT CAST(name 'namefield' AS text) AS "text(name)";

SELECT CAST(f1 AS char(10)) AS "char(text)" FROM TEXT_TBL;

SELECT CAST(f1 AS char(20)) AS "char(text)" FROM TEXT_TBL;

SELECT CAST(f1 AS char(10)) AS "char(varchar)" FROM VARCHAR_TBL;

SELECT CAST(name 'namefield' AS char(10)) AS "char(name)";

SELECT CAST(f1 AS varchar) AS "varchar(text)" FROM TEXT_TBL;

SELECT CAST(f1 AS varchar) AS "varchar(char)" FROM CHAR_TBL;

SELECT CAST(name 'namefield' AS varchar) AS "varchar(name)";

SELECT TRIM(BOTH FROM '  bunch o blanks  ') = 'bunch o blanks' AS "bunch o blanks";

SELECT TRIM(LEADING FROM '  bunch o blanks  ') = 'bunch o blanks  ' AS "bunch o blanks  ";

SELECT TRIM(TRAILING FROM '  bunch o blanks  ') = '  bunch o blanks' AS "  bunch o blanks";

SELECT TRIM(BOTH 'x' FROM 'xxxxxsome Xsxxxxx') = 'some Xs' AS "some Xs";

SELECT SUBSTRING('1234567890' FROM 3) = '34567890' AS "34567890";

SELECT SUBSTRING('1234567890' FROM 4 FOR 3) = '456' AS "456";

SELECT SUBSTRING('string' FROM 2 FOR 2147483646) AS "tring";

SELECT SUBSTRING('string' FROM -10 FOR 2147483646) AS "string";

SELECT SUBSTRING('string' FROM -10 FOR -2147483646) AS "error";

SELECT SUBSTRING('abcdefg' SIMILAR 'a#"(b_d)#"%' ESCAPE '#') AS "bcd";

SELECT SUBSTRING('abcdefg' FROM 'a#"(b_d)#"%' FOR '#') AS "bcd";

SELECT SUBSTRING('abcdefg' SIMILAR '#"(b_d)#"%' ESCAPE '#') IS NULL AS "True";

SELECT SUBSTRING('abcdefg' SIMILAR '%' ESCAPE NULL) IS NULL AS "True";

SELECT SUBSTRING(NULL SIMILAR '%' ESCAPE '#') IS NULL AS "True";

SELECT SUBSTRING('abcdefg' SIMILAR NULL ESCAPE '#') IS NULL AS "True";

SELECT SUBSTRING('abcdefg' SIMILAR 'a#"%#"g' ESCAPE '#') AS "bcdef";

SELECT SUBSTRING('abcdefg' SIMILAR 'a*#"%#"g*' ESCAPE '#') AS "abcdefg";

SELECT SUBSTRING('abcdefg' SIMILAR 'a|b#"%#"g' ESCAPE '#') AS "bcdef";

SELECT SUBSTRING('abcdefg' SIMILAR 'a#"%#"x|g' ESCAPE '#') AS "bcdef";

SELECT SUBSTRING('abcdefg' SIMILAR 'a#"%|ab#"g' ESCAPE '#') AS "bcdef";

SELECT SUBSTRING('abcdefg' SIMILAR 'a*#"%#"g*#"x' ESCAPE '#') AS "error";

SELECT SUBSTRING('abcdefg' SIMILAR 'a#"%g' ESCAPE '#') AS "bcdefg";

SELECT SUBSTRING('abcdefg' SIMILAR 'a%g' ESCAPE '#') AS "abcdefg";

SELECT SUBSTRING('abcdefg' FROM 'c.e') AS "cde";

SELECT SUBSTRING('abcdefg' FROM 'b(.*)f') AS "cde";

SELECT SUBSTRING('foo' FROM 'foo(bar)?') IS NULL AS t;

SELECT 'abcdefg' SIMILAR TO '_bcd%' AS true;

SELECT 'abcdefg' SIMILAR TO 'bcd%' AS false;

SELECT 'abcdefg' SIMILAR TO '_bcd#%' ESCAPE '#' AS false;

SELECT 'abcd%' SIMILAR TO '_bcd#%' ESCAPE '#' AS true;

SELECT 'abcdefg' SIMILAR TO '_bcd\%' AS false;

SELECT 'abcd\efg' SIMILAR TO '_bcd\%' ESCAPE '' AS true;

SELECT 'abcdefg' SIMILAR TO '_bcd%' ESCAPE NULL AS null;

SELECT 'abcdefg' SIMILAR TO '_bcd#%' ESCAPE '##' AS error;

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '_[_[:alpha:]_]_';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '%[%[:alnum:]%]%';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '.[.[:alnum:].].';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '$[$[:alnum:]$]$';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '()[([:alnum:](]()';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '^[^[:alnum:]^[^^][[^^]][\^][[\^]]\^]^';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '[]%][^]%][^%]%';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '[^^]^';

SELECT * FROM TEXT_TBL WHERE f1 SIMILAR TO '[|a]%' ESCAPE '|';

SELECT regexp_replace('1112223333', E'(\\d{3})(\\d{3})(\\d{4})', E'(\\1) \\2-\\3');

SELECT regexp_replace('foobarrbazz', E'(.)\\1', E'X\\&Y', 'g');

SELECT regexp_replace('foobarrbazz', E'(.)\\1', E'X\\\\Y', 'g');

SELECT regexp_replace('foobarrbazz', E'(.)\\1', E'X\\Y\\1Z\\');

SELECT regexp_replace('AAA   BBB   CCC   ', E'\\s+', ' ', 'g');

SELECT regexp_replace('AAA', '^|$', 'Z', 'g');

SELECT regexp_replace('AAA aaa', 'A+', 'Z', 'gi');

SELECT regexp_replace('AAA aaa', 'A+', 'Z', 'z');

SELECT regexp_replace('A PostgreSQL function', 'A|e|i|o|u', 'X', 1);

SELECT regexp_replace('A PostgreSQL function', 'A|e|i|o|u', 'X', 1, 2);

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 0, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 1, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 2, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 3, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 9, 'i');

SELECT regexp_replace('A PostgreSQL function', 'A|e|i|o|u', 'X', 7, 0, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, 1, 'g');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', -1, 0, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', 1, -1, 'i');

SELECT regexp_replace('A PostgreSQL function', 'a|e|i|o|u', 'X', '1');

SELECT regexp_count('123123123123123', '(12)3');

SELECT regexp_count('123123123123', '123', 1);

SELECT regexp_count('123123123123', '123', 3);

SELECT regexp_count('123123123123', '123', 33);

SELECT regexp_count('ABCABCABCABC', 'Abc', 1, '');

SELECT regexp_count('ABCABCABCABC', 'Abc', 1, 'i');

SELECT regexp_count('123123123123', '123', 0);

SELECT regexp_count('123123123123', '123', -3);

SELECT regexp_like('Steven', '^Ste(v|ph)en$');

SELECT regexp_like('a'||CHR(10)||'d', 'a.d', 'n');

SELECT regexp_like('a'||CHR(10)||'d', 'a.d', 's');

SELECT regexp_like('abc', ' a . c ', 'x');

SELECT regexp_like('abc', 'a.c', 'g');

SELECT regexp_instr('abcdefghi', 'd.f');

SELECT regexp_instr('abcdefghi', 'd.q');

SELECT regexp_instr('abcabcabc', 'a.c');

SELECT regexp_instr('abcabcabc', 'a.c', 2);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 3);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 4);

SELECT regexp_instr('abcabcabc', 'A.C', 1, 2, 0, 'i');

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 0);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 1);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 2);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 3);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 4);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 0, 'i', 5);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 0);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 1);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 2);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 3);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 4);

SELECT regexp_instr('1234567890', '(123)(4(56)(78))', 1, 1, 1, 'i', 5);

SELECT regexp_instr('foo', 'foo(bar)?', 1, 1, 0, '', 1);

SELECT regexp_instr('abcabcabc', 'a.c', 0, 1);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 0);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 1, -1);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 1, 2);

SELECT regexp_instr('abcabcabc', 'a.c', 1, 1, 0, 'g');

SELECT regexp_instr('abcabcabc', 'a.c', 1, 1, 0, '', -1);

SELECT regexp_substr('abcdefghi', 'd.f');

SELECT regexp_substr('abcdefghi', 'd.q') IS NULL AS t;

SELECT regexp_substr('abcabcabc', 'a.c');

SELECT regexp_substr('abcabcabc', 'a.c', 2);

SELECT regexp_substr('abcabcabc', 'a.c', 1, 3);

SELECT regexp_substr('abcabcabc', 'a.c', 1, 4) IS NULL AS t;

SELECT regexp_substr('abcabcabc', 'A.C', 1, 2, 'i');

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 0);

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 1);

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 2);

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 3);

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 4);

SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 5) IS NULL AS t;

SELECT regexp_substr('foo', 'foo(bar)?', 1, 1, '', 1) IS NULL AS t;

SELECT regexp_substr('abcabcabc', 'a.c', 0, 1);

SELECT regexp_substr('abcabcabc', 'a.c', 1, 0);

SELECT regexp_substr('abcabcabc', 'a.c', 1, 1, 'g');

SELECT regexp_substr('abcabcabc', 'a.c', 1, 1, '', -1);

SELECT regexp_matches('foobarbequebaz', $re$(bar)(beque)$re$);

SELECT regexp_matches('foObARbEqUEbAz', $re$(bar)(beque)$re$, 'i');

SELECT regexp_matches('foobarbequebazilbarfbonk', $re$(b[^b]+)(b[^b]+)$re$, 'g');

SELECT regexp_matches('foobarbequebaz', $re$(bar)(.*)(beque)$re$);

SELECT regexp_matches('foobarbequebaz', $re$(bar)(.+)(beque)$re$);

SELECT regexp_matches('foobarbequebaz', $re$(bar)(.+)?(beque)$re$);

SELECT regexp_matches('foobarbequebaz', $re$barbeque$re$);

SELECT regexp_matches('foo' || chr(10) || 'bar' || chr(10) || 'bequq' || chr(10) || 'baz', '^', 'mg');

SELECT regexp_matches('foo' || chr(10) || 'bar' || chr(10) || 'bequq' || chr(10) || 'baz', '$', 'mg');

SELECT regexp_matches('1' || chr(10) || '2' || chr(10) || '3' || chr(10) || '4' || chr(10), '^.?', 'mg');

SELECT regexp_matches(chr(10) || '1' || chr(10) || '2' || chr(10) || '3' || chr(10) || '4' || chr(10), '.?$', 'mg');

SELECT regexp_matches(chr(10) || '1' || chr(10) || '2' || chr(10) || '3' || chr(10) || '4', '.?$', 'mg');

SELECT regexp_matches('foobarbequebaz', $re$(bar)(beque)$re$, 'gz');

SELECT regexp_matches('foobarbequebaz', $re$(barbeque$re$);

SELECT regexp_matches('foobarbequebaz', $re$(bar)(beque){2,1}$re$);

SELECT foo, length(foo) FROM regexp_split_to_table('the quick brown fox jumps over the lazy dog', $re$\s+$re$) AS foo;

SELECT regexp_split_to_array('the quick brown fox jumps over the lazy dog', $re$\s+$re$);

SELECT foo, length(foo) FROM regexp_split_to_table('the quick brown fox jumps over the lazy dog', $re$\s*$re$) AS foo;

SELECT regexp_split_to_array('the quick brown fox jumps over the lazy dog', $re$\s*$re$);

SELECT foo, length(foo) FROM regexp_split_to_table('the quick brown fox jumps over the lazy dog', '') AS foo;

SELECT regexp_split_to_array('the quick brown fox jumps over the lazy dog', '');

SELECT foo, length(foo) FROM regexp_split_to_table('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'i') AS foo;

SELECT regexp_split_to_array('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'i');

SELECT foo, length(foo) FROM regexp_split_to_table('the quick brown fox jumps over the lazy dog', 'nomatch') AS foo;

SELECT regexp_split_to_array('the quick brown fox jumps over the lazy dog', 'nomatch');

SELECT regexp_split_to_array('123456','1');

SELECT regexp_split_to_array('123456','6');

SELECT regexp_split_to_array('123456','.');

SELECT regexp_split_to_array('123456','');

SELECT regexp_split_to_array('123456','(?:)');

SELECT regexp_split_to_array('1','');

SELECT foo, length(foo) FROM regexp_split_to_table('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'zippy') AS foo;

SELECT regexp_split_to_array('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'iz');

SELECT foo, length(foo) FROM regexp_split_to_table('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'g') AS foo;

SELECT regexp_split_to_array('thE QUick bROWn FOx jUMPs ovEr The lazy dOG', 'e', 'g');

SELECT POSITION('4' IN '1234567890') = '4' AS "4";

SELECT POSITION('5' IN '1234567890') = '5' AS "5";

SELECT POSITION('\x11'::bytea IN ''::bytea) = 0 AS "0";

SELECT POSITION('\x33'::bytea IN '\x1122'::bytea) = 0 AS "0";

SELECT POSITION(''::bytea IN '\x1122'::bytea) = 1 AS "1";

SELECT POSITION('\x22'::bytea IN '\x1122'::bytea) = 2 AS "2";

SELECT POSITION('\x5678'::bytea IN '\x1234567890'::bytea) = 3 AS "3";

SELECT OVERLAY('abcdef' PLACING '45' FROM 4) AS "abc45f";

SELECT OVERLAY('yabadoo' PLACING 'daba' FROM 5) AS "yabadaba";

SELECT OVERLAY('yabadoo' PLACING 'daba' FROM 5 FOR 0) AS "yabadabadoo";

SELECT OVERLAY('babosa' PLACING 'ubb' FROM 2 FOR 4) AS "bubba";

SELECT 'hawkeye' LIKE 'h%' AS "true";

SELECT 'hawkeye' NOT LIKE 'h%' AS "false";

SELECT 'hawkeye' LIKE 'H%' AS "false";

SELECT 'hawkeye' NOT LIKE 'H%' AS "true";

SELECT 'hawkeye' LIKE 'indio%' AS "false";

SELECT 'hawkeye' NOT LIKE 'indio%' AS "true";

SELECT 'hawkeye' LIKE 'h%eye' AS "true";

SELECT 'hawkeye' NOT LIKE 'h%eye' AS "false";

SELECT 'indio' LIKE '_ndio' AS "true";

SELECT 'indio' NOT LIKE '_ndio' AS "false";

SELECT 'indio' LIKE 'in__o' AS "true";

SELECT 'indio' NOT LIKE 'in__o' AS "false";

SELECT 'indio' LIKE 'in_o' AS "false";

SELECT 'indio' NOT LIKE 'in_o' AS "true";

SELECT 'abc'::name LIKE '_b_' AS "true";

SELECT 'abc'::name NOT LIKE '_b_' AS "false";

SELECT 'abc'::bytea LIKE '_b_'::bytea AS "true";

SELECT 'abc'::bytea NOT LIKE '_b_'::bytea AS "false";

SELECT 'hawkeye' LIKE 'h%' ESCAPE '#' AS "true";

SELECT 'hawkeye' NOT LIKE 'h%' ESCAPE '#' AS "false";

SELECT 'indio' LIKE 'ind_o' ESCAPE '$' AS "true";

SELECT 'indio' NOT LIKE 'ind_o' ESCAPE '$' AS "false";

SELECT 'h%' LIKE 'h#%' ESCAPE '#' AS "true";

SELECT 'h%' NOT LIKE 'h#%' ESCAPE '#' AS "false";

SELECT 'h%wkeye' LIKE 'h#%' ESCAPE '#' AS "false";

SELECT 'h%wkeye' NOT LIKE 'h#%' ESCAPE '#' AS "true";

SELECT 'h%wkeye' LIKE 'h#%%' ESCAPE '#' AS "true";

SELECT 'h%wkeye' NOT LIKE 'h#%%' ESCAPE '#' AS "false";

SELECT 'h%awkeye' LIKE 'h#%a%k%e' ESCAPE '#' AS "true";

SELECT 'h%awkeye' NOT LIKE 'h#%a%k%e' ESCAPE '#' AS "false";

SELECT 'indio' LIKE '_ndio' ESCAPE '$' AS "true";

SELECT 'indio' NOT LIKE '_ndio' ESCAPE '$' AS "false";

SELECT 'i_dio' LIKE 'i$_d_o' ESCAPE '$' AS "true";

SELECT 'i_dio' NOT LIKE 'i$_d_o' ESCAPE '$' AS "false";

SELECT 'i_dio' LIKE 'i$_nd_o' ESCAPE '$' AS "false";

SELECT 'i_dio' NOT LIKE 'i$_nd_o' ESCAPE '$' AS "true";

SELECT 'i_dio' LIKE 'i$_d%o' ESCAPE '$' AS "true";

SELECT 'i_dio' NOT LIKE 'i$_d%o' ESCAPE '$' AS "false";

SELECT 'a_c'::bytea LIKE 'a$__'::bytea ESCAPE '$'::bytea AS "true";

SELECT 'a_c'::bytea NOT LIKE 'a$__'::bytea ESCAPE '$'::bytea AS "false";

SELECT 'maca' LIKE 'm%aca' ESCAPE '%' AS "true";

SELECT 'maca' NOT LIKE 'm%aca' ESCAPE '%' AS "false";

SELECT 'ma%a' LIKE 'm%a%%a' ESCAPE '%' AS "true";

SELECT 'ma%a' NOT LIKE 'm%a%%a' ESCAPE '%' AS "false";

SELECT 'bear' LIKE 'b_ear' ESCAPE '_' AS "true";

SELECT 'bear' NOT LIKE 'b_ear' ESCAPE '_' AS "false";

SELECT 'be_r' LIKE 'b_e__r' ESCAPE '_' AS "true";

SELECT 'be_r' NOT LIKE 'b_e__r' ESCAPE '_' AS "false";

SELECT 'be_r' LIKE '__e__r' ESCAPE '_' AS "false";

SELECT 'be_r' NOT LIKE '__e__r' ESCAPE '_' AS "true";

SELECT 'hawkeye' ILIKE 'h%' AS "true";

SELECT 'hawkeye' NOT ILIKE 'h%' AS "false";

SELECT 'hawkeye' ILIKE 'H%' AS "true";

SELECT 'hawkeye' NOT ILIKE 'H%' AS "false";

SELECT 'hawkeye' ILIKE 'H%Eye' AS "true";

SELECT 'hawkeye' NOT ILIKE 'H%Eye' AS "false";

SELECT 'Hawkeye' ILIKE 'h%' AS "true";

SELECT 'Hawkeye' NOT ILIKE 'h%' AS "false";

SELECT 'ABC'::name ILIKE '_b_' AS "true";

SELECT 'ABC'::name NOT ILIKE '_b_' AS "false";

SELECT 'foo' LIKE '_%' as t, 'f' LIKE '_%' as t, '' LIKE '_%' as f;

SELECT 'foo' LIKE '%_' as t, 'f' LIKE '%_' as t, '' LIKE '%_' as f;

SELECT 'foo' LIKE '__%' as t, 'foo' LIKE '___%' as t, 'foo' LIKE '____%' as f;

SELECT 'foo' LIKE '%__' as t, 'foo' LIKE '%___' as t, 'foo' LIKE '%____' as f;

SELECT 'jack' LIKE '%____%' AS t;

CREATE TABLE texttest (a text PRIMARY KEY, b int);

SELECT * FROM texttest WHERE a LIKE '%1%';

CREATE TABLE byteatest (a bytea PRIMARY KEY, b int);

SELECT * FROM byteatest WHERE a LIKE '%1%';

DROP TABLE texttest, byteatest;

SELECT 'unknown' || ' and unknown' AS "Concat unknown types";

SELECT text 'text' || ' and unknown' AS "Concat text to unknown type";

SELECT char(20) 'characters' || ' and text' AS "Concat char to unknown type";

SELECT text 'text' || char(20) ' and characters' AS "Concat text to char";

SELECT text 'text' || varchar ' and varchar' AS "Concat text to varchar";

CREATE TABLE toasttest(f1 text);

insert into toasttest values(repeat('1234567890',10000));

insert into toasttest values(repeat('1234567890',10000));

alter table toasttest alter column f1 set storage external;

insert into toasttest values(repeat('1234567890',10000));

insert into toasttest values(repeat('1234567890',10000));

SELECT substr(f1, -1, 5) from toasttest;

SELECT substr(f1, 5, -1) from toasttest;

SELECT substr(f1, 99995) from toasttest;

SELECT substr(f1, 99995, 10) from toasttest;

TRUNCATE TABLE toasttest;

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

SELECT pg_relation_size(reltoastrelid) = 0 AS is_empty
  FROM pg_class where relname = 'toasttest';

TRUNCATE TABLE toasttest;

ALTER TABLE toasttest set (toast_tuple_target = 4080);

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

INSERT INTO toasttest values (repeat('1234567890',300));

SELECT pg_relation_size(reltoastrelid) = 0 AS is_empty
  FROM pg_class where relname = 'toasttest';

DROP TABLE toasttest;

CREATE TABLE toasttest(f1 bytea);

insert into toasttest values(decode(repeat('1234567890',10000),'escape'));

insert into toasttest values(decode(repeat('1234567890',10000),'escape'));

alter table toasttest alter column f1 set storage external;

insert into toasttest values(decode(repeat('1234567890',10000),'escape'));

insert into toasttest values(decode(repeat('1234567890',10000),'escape'));

SELECT substr(f1, -1, 5) from toasttest;

SELECT substr(f1, 5, -1) from toasttest;

SELECT substr(f1, 99995) from toasttest;

SELECT substr(f1, 99995, 10) from toasttest;

DROP TABLE toasttest;

CREATE TABLE toasttest (c char(4096));

INSERT INTO toasttest VALUES('x');

SELECT length(c), c::text FROM toasttest;

SELECT c FROM toasttest;

DROP TABLE toasttest;

CREATE TABLE toasttest (f1 text, f2 text);

ALTER TABLE toasttest SET (toast_tuple_target = 128);

ALTER TABLE toasttest ALTER COLUMN f1 SET STORAGE EXTERNAL;

ALTER TABLE toasttest ALTER COLUMN f2 SET STORAGE EXTERNAL;

INSERT INTO toasttest values(repeat('1234', 1000), repeat('5678', 30));

SELECT reltoastrelid::regclass AS reltoastname FROM pg_class
  WHERE oid = 'toasttest'::regclass ;

SELECT substr(f1, 5, 10) AS f1_data, substr(f2, 5, 10) AS f2_data
  FROM toasttest;

SELECT pg_column_compression(f1) AS f1_comp, pg_column_compression(f2) AS f2_comp
  FROM toasttest;

DROP TABLE toasttest;

SELECT length('abcdef') AS "length_6";

SELECT strpos('abcdef', 'cd') AS "pos_3";

SELECT strpos('abcdef', 'xy') AS "pos_0";

SELECT strpos('abcdef', '') AS "pos_1";

SELECT strpos('', 'xy') AS "pos_0";

SELECT strpos('', '') AS "pos_1";

SELECT replace('abcdef', 'de', '45') AS "abc45f";

SELECT replace('yabadabadoo', 'ba', '123') AS "ya123da123doo";

SELECT replace('yabadoo', 'bad', '') AS "yaoo";

select split_part('','@',1) AS "empty string";

select split_part('','@',-1) AS "empty string";

select split_part('joeuser@mydatabase','',1) AS "joeuser@mydatabase";

select split_part('joeuser@mydatabase','',2) AS "empty string";

select split_part('joeuser@mydatabase','',-1) AS "joeuser@mydatabase";

select split_part('joeuser@mydatabase','',-2) AS "empty string";

select split_part('joeuser@mydatabase','@',0) AS "an error";

select split_part('joeuser@mydatabase','@@',1) AS "joeuser@mydatabase";

select split_part('joeuser@mydatabase','@@',2) AS "empty string";

select split_part('joeuser@mydatabase','@',1) AS "joeuser";

select split_part('joeuser@mydatabase','@',2) AS "mydatabase";

select split_part('joeuser@mydatabase','@',3) AS "empty string";

select split_part('@joeuser@mydatabase@','@',2) AS "joeuser";

select split_part('joeuser@mydatabase','@',-1) AS "mydatabase";

select split_part('joeuser@mydatabase','@',-2) AS "joeuser";

select split_part('joeuser@mydatabase','@',-3) AS "empty string";

select split_part('@joeuser@mydatabase@','@',-2) AS "mydatabase";

select to_bin(-1234) AS "11111111111111111111101100101110";

select to_bin(-1234::bigint);

select to_bin(256*256*256 - 1) AS "111111111111111111111111";

select to_bin(256::bigint*256::bigint*256::bigint*256::bigint - 1) AS "11111111111111111111111111111111";

select to_oct(-1234) AS "37777775456";

select to_oct(-1234::bigint) AS "1777777777777777775456";

select to_oct(256*256*256 - 1) AS "77777777";

select to_oct(256::bigint*256::bigint*256::bigint*256::bigint - 1) AS "37777777777";

select to_hex(-1234) AS "fffffb2e";

select to_hex(-1234::bigint) AS "fffffffffffffb2e";

select to_hex(256*256*256 - 1) AS "ffffff";

select to_hex(256::bigint*256::bigint*256::bigint*256::bigint - 1) AS "ffffffff";

SET bytea_output TO hex;

SELECT sha224('');

SELECT sha224('The quick brown fox jumps over the lazy dog.');

SELECT sha256('');

SELECT sha256('The quick brown fox jumps over the lazy dog.');

SELECT sha384('');

SELECT sha384('The quick brown fox jumps over the lazy dog.');

SELECT sha512('');

SELECT sha512('The quick brown fox jumps over the lazy dog.');

SELECT crc32('');

SELECT crc32('The quick brown fox jumps over the lazy dog.');

SELECT crc32c('');

SELECT crc32c('The quick brown fox jumps over the lazy dog.');

SELECT crc32c(repeat('A', 127)::bytea);

SELECT crc32c(repeat('A', 128)::bytea);

SELECT crc32c(repeat('A', 129)::bytea);

SELECT crc32c(repeat('A', 800)::bytea);

SELECT encode('\x1234567890abcdef00', 'hex');

SELECT decode('1234567890abcdef00', 'hex');

SELECT encode(('\x' || repeat('1234567890abcdef0001', 7))::bytea, 'base64');

SELECT decode(encode(('\x' || repeat('1234567890abcdef0001', 7))::bytea,
                     'base64'), 'base64');

SELECT encode('\x1234567890abcdef00', 'escape');

SELECT decode(encode('\x1234567890abcdef00', 'escape'), 'escape');

SET bytea_output TO hex;

SELECT encode('\x69b73eff', 'base64url');

SELECT decode('abc-_w', 'base64url');

SELECT decode(encode('\x1234567890abcdef00', 'base64url'), 'base64url');

SELECT encode('', 'base64url');

SELECT decode('', 'base64url');

SELECT encode('\x01', 'base64url');

SELECT decode('AQ', 'base64url');

SELECT encode('\x0102'::bytea, 'base64url');

SELECT decode('AQI', 'base64url');

SELECT encode('\x010203'::bytea, 'base64url');

SELECT decode('AQID', 'base64url');

SELECT encode('\xdeadbeef'::bytea, 'base64url');

SELECT decode('3q2-7w', 'base64url');

SELECT encode(decode(encode(E'\\x', 'base64url'), 'base64url'), 'base64url');

SELECT encode(decode(encode(E'\\x00', 'base64url'), 'base64url'), 'base64url');

SELECT encode(decode(encode(E'\\x0001', 'base64url'), 'base64url'), 'base64url');

SELECT encode(decode(encode(E'\\x000102', 'base64url'), 'base64url'), 'base64url');

SELECT encode(decode(encode(E'\\x00010203', 'base64url'), 'base64url'), 'base64url');

SELECT decode('QQ@=', 'base64url');

SELECT decode('QQ', 'base64url');

SELECT decode('QQI', 'base64url');

SELECT decode('QQIDQ', 'base64url');

SELECT decode('=QQQ', 'base64url');

SELECT decode('abc-_w==', 'base64url');

SELECT get_bit('\x1234567890abcdef00'::bytea, 43);

SELECT get_bit('\x1234567890abcdef00'::bytea, 99);

SELECT set_bit('\x1234567890abcdef00'::bytea, 43, 0);

SELECT set_bit('\x1234567890abcdef00'::bytea, 99, 0);

SELECT get_byte('\x1234567890abcdef00'::bytea, 3);

SELECT get_byte('\x1234567890abcdef00'::bytea, 99);

SELECT set_byte('\x1234567890abcdef00'::bytea, 7, 11);

SELECT set_byte('\x1234567890abcdef00'::bytea, 99, 11);

SELECT 0x1234::int2::bytea AS "\x1234", (-0x1234)::int2::bytea AS "\xedcc";

SELECT 0x12345678::int4::bytea AS "\x12345678", (-0x12345678)::int4::bytea AS "\xedcba988";

SELECT 0x1122334455667788::int8::bytea AS "\x1122334455667788",
       (-0x1122334455667788)::int8::bytea AS "\xeeddccbbaa998878";

SELECT ''::bytea::int2 AS "0";

SELECT '\x12'::bytea::int2 AS "18";

SELECT '\x1234'::bytea::int2 AS "4460";

SELECT '\x123456'::bytea::int2;

SELECT ''::bytea::int4 AS "0";

SELECT '\x12'::bytea::int4 AS "18";

SELECT '\x12345678'::bytea::int4 AS "305419896";

SELECT '\x123456789A'::bytea::int4;

SELECT ''::bytea::int8 AS "0";

SELECT '\x12'::bytea::int8 AS "18";

SELECT '\x1122334455667788'::bytea::int8 AS "1234605616436508552";

SELECT '\x112233445566778899'::bytea::int8;

SELECT '\x8000'::bytea::int2 AS "-32768", '\x7FFF'::bytea::int2 AS "32767";

SELECT '\x80000000'::bytea::int4 AS "-2147483648", '\x7FFFFFFF'::bytea::int4 AS "2147483647";

SELECT '\x8000000000000000'::bytea::int8 AS "-9223372036854775808",
       '\x7FFFFFFFFFFFFFFF'::bytea::int8 AS "9223372036854775807";

set escape_string_warning = off;

set standard_conforming_strings = off;

show escape_string_warning;

show standard_conforming_strings;

set escape_string_warning = on;

set standard_conforming_strings = on;

show escape_string_warning;

show standard_conforming_strings;

select 'a\bcd' as f1, 'a\b''cd' as f2, 'a\b''''cd' as f3, 'abcd\'   as f4, 'ab\''cd' as f5, '\\' as f6;

set standard_conforming_strings = off;

reset standard_conforming_strings;

SET bytea_output TO escape;

SELECT initcap('hi THOMAS');

SELECT lpad('hi', 5, 'xy');

SELECT lpad('hi', 5);

SELECT lpad('hi', -5, 'xy');

SELECT lpad('hello', 2);

SELECT lpad('hi', 5, '');

SELECT rpad('hi', 5, 'xy');

SELECT rpad('hi', 5);

SELECT rpad('hi', -5, 'xy');

SELECT rpad('hello', 2);

SELECT rpad('hi', 5, '');

SELECT ltrim('zzzytrim', 'xyz');

SELECT translate('', '14', 'ax');

SELECT translate('12345', '14', 'ax');

SELECT translate('12345', '134', 'a');

SELECT ascii('x');

SELECT ascii('');

SELECT chr(65);

SELECT chr(0);

SELECT repeat('Pg', 4);

SELECT repeat('Pg', -4);

SELECT SUBSTRING('1234567890'::bytea FROM 3) "34567890";

SELECT SUBSTRING('1234567890'::bytea FROM 4 FOR 3) AS "456";

SELECT SUBSTRING('string'::bytea FROM 2 FOR 2147483646) AS "tring";

SELECT SUBSTRING('string'::bytea FROM -10 FOR 2147483646) AS "string";

SELECT SUBSTRING('string'::bytea FROM -10 FOR -2147483646) AS "error";

SELECT trim(E'\\000'::bytea from E'\\000Tom\\000'::bytea);

SELECT trim(leading E'\\000'::bytea from E'\\000Tom\\000'::bytea);

SELECT trim(trailing E'\\000'::bytea from E'\\000Tom\\000'::bytea);

SELECT btrim(E'\\000trim\\000'::bytea, E'\\000'::bytea);

SELECT btrim(''::bytea, E'\\000'::bytea);

SELECT btrim(E'\\000trim\\000'::bytea, ''::bytea);

SELECT encode(overlay(E'Th\\000omas'::bytea placing E'Th\\001omas'::bytea from 2),'escape');

SELECT encode(overlay(E'Th\\000omas'::bytea placing E'\\002\\003'::bytea from 8),'escape');

SELECT encode(overlay(E'Th\\000omas'::bytea placing E'\\002\\003'::bytea from 5 for 3),'escape');

SELECT bit_count('\x1234567890'::bytea);

SELECT unistr('\0064at\+0000610');

SELECT unistr('d\u0061t\U000000610');

SELECT unistr('a\\b');

SELECT unistr('wrong: \db99');

SELECT unistr('wrong: \db99\0061');

SELECT unistr('wrong: \+00db99\+000061');

SELECT unistr('wrong: \+2FFFFF');

SELECT unistr('wrong: \udb99\u0061');

SELECT unistr('wrong: \U0000db99\U00000061');

SELECT unistr('wrong: \U002FFFFF');

SELECT unistr('wrong: \xyz');
