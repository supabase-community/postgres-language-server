SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

PREPARE q1 AS SELECT 1 AS a;

EXECUTE q1;

SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

PREPARE q1 AS SELECT 2;

DEALLOCATE q1;

PREPARE q1 AS SELECT 2;

EXECUTE q1;

PREPARE q2 AS SELECT 2 AS b;

SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

DEALLOCATE PREPARE q1;

SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

DEALLOCATE PREPARE q2;

SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements;

PREPARE q2(text) AS
	SELECT datname, datistemplate, datallowconn
	FROM pg_database WHERE datname = $1;

EXECUTE q2('postgres');

PREPARE q3(text, int, float, boolean, smallint) AS
	SELECT * FROM tenk1 WHERE string4 = $1 AND (four = $2 OR
	ten = $3::bigint OR true = $4 OR odd = $5::int)
	ORDER BY unique1;

EXECUTE q3('AAAAxx', 5::smallint, 10.5::float, false, 4::bigint);

EXECUTE q3('bool');

EXECUTE q3('bytea', 5::smallint, 10.5::float, false, 4::bigint, true);

EXECUTE q3(5::smallint, 10.5::float, false, 4::bigint, 'bytea');

PREPARE q4(nonexistenttype) AS SELECT $1;

PREPARE q5(int, text) AS
	SELECT * FROM tenk1 WHERE unique1 = $1 OR stringu1 = $2
	ORDER BY unique1;

CREATE TEMPORARY TABLE q5_prep_results AS EXECUTE q5(200, 'DTAAAA');

SELECT * FROM q5_prep_results;

CREATE TEMPORARY TABLE q5_prep_nodata AS EXECUTE q5(200, 'DTAAAA')
    WITH NO DATA;

SELECT * FROM q5_prep_nodata;

PREPARE q6 AS
    SELECT * FROM tenk1 WHERE unique1 = $1 AND stringu1 = $2;

PREPARE q7(unknown) AS
    SELECT * FROM road WHERE thepath = $1;

UPDATE tenk1 SET stringu1 = $2 WHERE unique1 = $1;

SELECT name, statement, parameter_types, result_types FROM pg_prepared_statements
    ORDER BY name;

DEALLOCATE ALL;

SELECT name, statement, parameter_types FROM pg_prepared_statements
    ORDER BY name;
