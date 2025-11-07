SELECT 1 AS one;

SELECT true AS true;

SELECT false AS false;

SELECT bool 't' AS true;

SELECT bool '   f           ' AS false;

SELECT bool 'true' AS true;

SELECT bool 'test' AS error;

SELECT bool 'false' AS false;

SELECT bool 'foo' AS error;

SELECT bool 'y' AS true;

SELECT bool 'yes' AS true;

SELECT bool 'yeah' AS error;

SELECT bool 'n' AS false;

SELECT bool 'no' AS false;

SELECT bool 'nay' AS error;

SELECT bool 'on' AS true;

SELECT bool 'off' AS false;

SELECT bool 'of' AS false;

SELECT bool 'o' AS error;

SELECT bool 'on_' AS error;

SELECT bool 'off_' AS error;

SELECT bool '1' AS true;

SELECT bool '11' AS error;

SELECT bool '0' AS false;

SELECT bool '000' AS error;

SELECT bool '' AS error;

SELECT pg_input_is_valid('true', 'bool');

SELECT pg_input_is_valid('asdf', 'bool');

SELECT * FROM pg_input_error_info('junk', 'bool');

SELECT bool 't' or bool 'f' AS true;

SELECT bool 't' and bool 'f' AS false;

SELECT not bool 'f' AS true;

SELECT bool 't' = bool 'f' AS false;

SELECT bool 't' <> bool 'f' AS true;

SELECT bool 't' > bool 'f' AS true;

SELECT bool 't' >= bool 'f' AS true;

SELECT bool 'f' < bool 't' AS true;

SELECT bool 'f' <= bool 't' AS true;

SELECT 'TrUe'::text::boolean AS true, 'fAlse'::text::boolean AS false;

SELECT '    true   '::text::boolean AS true,
       '     FALSE'::text::boolean AS false;

SELECT true::boolean::text AS true, false::boolean::text AS false;

SELECT '  tru e '::text::boolean AS invalid;

SELECT ''::text::boolean AS invalid;

CREATE TABLE BOOLTBL1 (f1 bool);

INSERT INTO BOOLTBL1 (f1) VALUES (bool 't');

INSERT INTO BOOLTBL1 (f1) VALUES (bool 'True');

INSERT INTO BOOLTBL1 (f1) VALUES (bool 'true');

SELECT BOOLTBL1.* FROM BOOLTBL1;

SELECT BOOLTBL1.*
   FROM BOOLTBL1
   WHERE f1 = bool 'true';

SELECT BOOLTBL1.*
   FROM BOOLTBL1
   WHERE f1 <> bool 'false';

SELECT BOOLTBL1.*
   FROM BOOLTBL1
   WHERE booleq(bool 'false', f1);

INSERT INTO BOOLTBL1 (f1) VALUES (bool 'f');

SELECT BOOLTBL1.*
   FROM BOOLTBL1
   WHERE f1 = bool 'false';

CREATE TABLE BOOLTBL2 (f1 bool);

INSERT INTO BOOLTBL2 (f1) VALUES (bool 'f');

INSERT INTO BOOLTBL2 (f1) VALUES (bool 'false');

INSERT INTO BOOLTBL2 (f1) VALUES (bool 'False');

INSERT INTO BOOLTBL2 (f1) VALUES (bool 'FALSE');

INSERT INTO BOOLTBL2 (f1)
   VALUES (bool 'XXX');

SELECT BOOLTBL2.* FROM BOOLTBL2;

SELECT BOOLTBL1.*, BOOLTBL2.*
   FROM BOOLTBL1, BOOLTBL2
   WHERE BOOLTBL2.f1 <> BOOLTBL1.f1;

SELECT BOOLTBL1.*, BOOLTBL2.*
   FROM BOOLTBL1, BOOLTBL2
   WHERE boolne(BOOLTBL2.f1,BOOLTBL1.f1);

SELECT BOOLTBL1.*, BOOLTBL2.*
   FROM BOOLTBL1, BOOLTBL2
   WHERE BOOLTBL2.f1 = BOOLTBL1.f1 and BOOLTBL1.f1 = bool 'false';

SELECT BOOLTBL1.*, BOOLTBL2.*
   FROM BOOLTBL1, BOOLTBL2
   WHERE BOOLTBL2.f1 = BOOLTBL1.f1 or BOOLTBL1.f1 = bool 'true'
   ORDER BY BOOLTBL1.f1, BOOLTBL2.f1;

SELECT f1
   FROM BOOLTBL1
   WHERE f1 IS TRUE;

SELECT f1
   FROM BOOLTBL1
   WHERE f1 IS NOT FALSE;

SELECT f1
   FROM BOOLTBL1
   WHERE f1 IS FALSE;

SELECT f1
   FROM BOOLTBL1
   WHERE f1 IS NOT TRUE;

SELECT f1
   FROM BOOLTBL2
   WHERE f1 IS TRUE;

SELECT f1
   FROM BOOLTBL2
   WHERE f1 IS NOT FALSE;

SELECT f1
   FROM BOOLTBL2
   WHERE f1 IS FALSE;

SELECT f1
   FROM BOOLTBL2
   WHERE f1 IS NOT TRUE;

CREATE TABLE BOOLTBL3 (d text, b bool, o int);

INSERT INTO BOOLTBL3 (d, b, o) VALUES ('true', true, 1);

INSERT INTO BOOLTBL3 (d, b, o) VALUES ('false', false, 2);

INSERT INTO BOOLTBL3 (d, b, o) VALUES ('null', null, 3);

SELECT
    d,
    b IS TRUE AS istrue,
    b IS NOT TRUE AS isnottrue,
    b IS FALSE AS isfalse,
    b IS NOT FALSE AS isnotfalse,
    b IS UNKNOWN AS isunknown,
    b IS NOT UNKNOWN AS isnotunknown
FROM booltbl3 ORDER BY o;

CREATE TABLE booltbl4(isfalse bool, istrue bool, isnul bool);

INSERT INTO booltbl4 VALUES (false, true, null);

SELECT istrue AND isnul AND istrue FROM booltbl4;

SELECT istrue AND istrue AND isnul FROM booltbl4;

SELECT isnul AND istrue AND istrue FROM booltbl4;

SELECT isfalse AND isnul AND istrue FROM booltbl4;

SELECT istrue AND isfalse AND isnul FROM booltbl4;

SELECT isnul AND istrue AND isfalse FROM booltbl4;

SELECT isfalse OR isnul OR isfalse FROM booltbl4;

SELECT isfalse OR isfalse OR isnul FROM booltbl4;

SELECT isnul OR isfalse OR isfalse FROM booltbl4;

SELECT isfalse OR isnul OR istrue FROM booltbl4;

SELECT istrue OR isfalse OR isnul FROM booltbl4;

SELECT isnul OR istrue OR isfalse FROM booltbl4;

SELECT 0::boolean;

SELECT 1::boolean;

SELECT 2::boolean;

DROP TABLE  BOOLTBL1;

DROP TABLE  BOOLTBL2;

DROP TABLE  BOOLTBL3;

DROP TABLE  BOOLTBL4;
