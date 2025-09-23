SELECT char 'c' = char 'c' AS true;

CREATE TEMP TABLE CHAR_TBL(f1 char);

INSERT INTO CHAR_TBL (f1) VALUES ('a');

INSERT INTO CHAR_TBL (f1) VALUES ('A');

INSERT INTO CHAR_TBL (f1) VALUES ('1');

INSERT INTO CHAR_TBL (f1) VALUES (2);

INSERT INTO CHAR_TBL (f1) VALUES ('3');

INSERT INTO CHAR_TBL (f1) VALUES ('');

INSERT INTO CHAR_TBL (f1) VALUES ('cd');

INSERT INTO CHAR_TBL (f1) VALUES ('c     ');

SELECT * FROM CHAR_TBL;

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 <> 'a';

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 = 'a';

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 < 'a';

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 <= 'a';

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 > 'a';

SELECT c.*
   FROM CHAR_TBL c
   WHERE c.f1 >= 'a';

DROP TABLE CHAR_TBL;

INSERT INTO CHAR_TBL (f1) VALUES ('abcde');

SELECT * FROM CHAR_TBL;

SELECT pg_input_is_valid('abcd  ', 'char(4)');

SELECT pg_input_is_valid('abcde', 'char(4)');

SELECT * FROM pg_input_error_info('abcde', 'char(4)');

SELECT 'a'::"char";

SELECT '\101'::"char";

SELECT '\377'::"char";

SELECT 'a'::"char"::text;

SELECT '\377'::"char"::text;

SELECT '\000'::"char"::text;

SELECT 'a'::text::"char";

SELECT '\377'::text::"char";

SELECT ''::text::"char";
