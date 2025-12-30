CREATE ROLE regress_regrole_test;

SELECT regoper('||/');

SELECT regoperator('+(int4,int4)');

SELECT regproc('now');

SELECT regprocedure('abs(numeric)');

SELECT regclass('pg_class');

SELECT regtype('int4');

SELECT regcollation('"POSIX"');

SELECT to_regoper('||/');

SELECT to_regoperator('+(int4,int4)');

SELECT to_regproc('now');

SELECT to_regprocedure('abs(numeric)');

SELECT to_regclass('pg_class');

SELECT to_regtype('int4');

SELECT to_regcollation('"POSIX"');

SELECT regoper('pg_catalog.||/');

SELECT regoperator('pg_catalog.+(int4,int4)');

SELECT regproc('pg_catalog.now');

SELECT regprocedure('pg_catalog.abs(numeric)');

SELECT regclass('pg_catalog.pg_class');

SELECT regtype('pg_catalog.int4');

SELECT regcollation('pg_catalog."POSIX"');

SELECT to_regoper('pg_catalog.||/');

SELECT to_regproc('pg_catalog.now');

SELECT to_regprocedure('pg_catalog.abs(numeric)');

SELECT to_regclass('pg_catalog.pg_class');

SELECT to_regtype('pg_catalog.int4');

SELECT to_regcollation('pg_catalog."POSIX"');

SELECT regrole('regress_regrole_test');

SELECT regrole('"regress_regrole_test"');

SELECT regnamespace('pg_catalog');

SELECT regnamespace('"pg_catalog"');

SELECT regdatabase('template1');

SELECT regdatabase('"template1"');

SELECT to_regrole('regress_regrole_test');

SELECT to_regrole('"regress_regrole_test"');

SELECT to_regnamespace('pg_catalog');

SELECT to_regnamespace('"pg_catalog"');

SELECT to_regdatabase('template1');

SELECT to_regdatabase('"template1"');

SELECT regproc('-')::oid;

SELECT regprocedure('-')::oid;

SELECT regclass('-')::oid;

SELECT regcollation('-')::oid;

SELECT regtype('-')::oid;

SELECT regconfig('-')::oid;

SELECT regdictionary('-')::oid;

SELECT regrole('-')::oid;

SELECT regnamespace('-')::oid;

SELECT regdatabase('-')::oid;

SELECT to_regproc('-')::oid;

SELECT to_regprocedure('-')::oid;

SELECT to_regclass('-')::oid;

SELECT to_regcollation('-')::oid;

SELECT to_regtype('-')::oid;

SELECT to_regrole('-')::oid;

SELECT to_regnamespace('-')::oid;

SELECT to_regdatabase('-')::oid;

CREATE TABLE regrole_test (rolid OID DEFAULT 'regress_regrole_test'::regrole);

CREATE TABLE regdatabase_test (datid OID DEFAULT 'template1'::regdatabase);

DROP ROLE regress_regrole_test;

SELECT regoper('||//');

SELECT regoperator('++(int4,int4)');

SELECT regproc('know');

SELECT regprocedure('absinthe(numeric)');

SELECT regclass('pg_classes');

SELECT regtype('int3');

SELECT regoper('ng_catalog.||/');

SELECT regoperator('ng_catalog.+(int4,int4)');

SELECT regproc('ng_catalog.now');

SELECT regprocedure('ng_catalog.abs(numeric)');

SELECT regclass('ng_catalog.pg_class');

SELECT regtype('ng_catalog.int4');

SELECT regcollation('ng_catalog."POSIX"');

SELECT regrole('regress_regrole_test');

SELECT regrole('"regress_regrole_test"');

SELECT regrole('Nonexistent');

SELECT regrole('"Nonexistent"');

SELECT regrole('foo.bar');

SELECT regnamespace('Nonexistent');

SELECT regnamespace('"Nonexistent"');

SELECT regnamespace('foo.bar');

SELECT regdatabase('Nonexistent');

SELECT regdatabase('"Nonexistent"');

SELECT regdatabase('foo.bar');

SELECT to_regoper('||//');

SELECT to_regoperator('++(int4,int4)');

SELECT to_regproc('know');

SELECT to_regprocedure('absinthe(numeric)');

SELECT to_regclass('pg_classes');

SELECT to_regtype('int3');

SELECT to_regcollation('notacollation');

SELECT to_regoper('ng_catalog.||/');

SELECT to_regoperator('ng_catalog.+(int4,int4)');

SELECT to_regproc('ng_catalog.now');

SELECT to_regprocedure('ng_catalog.abs(numeric)');

SELECT to_regclass('ng_catalog.pg_class');

SELECT to_regtype('ng_catalog.int4');

SELECT to_regcollation('ng_catalog."POSIX"');

SELECT to_regrole('regress_regrole_test');

SELECT to_regrole('"regress_regrole_test"');

SELECT to_regrole('foo.bar');

SELECT to_regrole('Nonexistent');

SELECT to_regrole('"Nonexistent"');

SELECT to_regrole('foo.bar');

SELECT to_regnamespace('Nonexistent');

SELECT to_regnamespace('"Nonexistent"');

SELECT to_regnamespace('foo.bar');

SELECT to_regdatabase('Nonexistent');

SELECT to_regdatabase('"Nonexistent"');

SELECT to_regdatabase('foo.bar');

SELECT to_regtypemod('text');

SELECT to_regtypemod('timestamp(4)');

SELECT to_regtypemod('no_such_type(4)');

SELECT format_type(to_regtype('varchar(32)'), to_regtypemod('varchar(32)'));

SELECT format_type(to_regtype('bit'), to_regtypemod('bit'));

SELECT format_type(to_regtype('"bit"'), to_regtypemod('"bit"'));

SELECT * FROM pg_input_error_info('ng_catalog.pg_class', 'regclass');

SELECT pg_input_is_valid('ng_catalog."POSIX"', 'regcollation');

SELECT * FROM pg_input_error_info('no_such_config', 'regconfig');

SELECT * FROM pg_input_error_info('no_such_dictionary', 'regdictionary');

SELECT * FROM pg_input_error_info('Nonexistent', 'regnamespace');

SELECT * FROM pg_input_error_info('ng_catalog.||/', 'regoper');

SELECT * FROM pg_input_error_info('-', 'regoper');

SELECT * FROM pg_input_error_info('ng_catalog.+(int4,int4)', 'regoperator');

SELECT * FROM pg_input_error_info('-', 'regoperator');

SELECT * FROM pg_input_error_info('ng_catalog.now', 'regproc');

SELECT * FROM pg_input_error_info('ng_catalog.abs(numeric)', 'regprocedure');

SELECT * FROM pg_input_error_info('ng_catalog.abs(numeric', 'regprocedure');

SELECT * FROM pg_input_error_info('regress_regrole_test', 'regrole');

SELECT * FROM pg_input_error_info('no_such_type', 'regtype');

SELECT * FROM pg_input_error_info('Nonexistent', 'regdatabase');

SELECT * FROM pg_input_error_info('incorrect type name syntax', 'regtype');

SELECT * FROM pg_input_error_info('numeric(1,2,3)', 'regtype');

SELECT * FROM pg_input_error_info('way.too.many.names', 'regtype');

SELECT * FROM pg_input_error_info('no_such_catalog.schema.name', 'regtype');
