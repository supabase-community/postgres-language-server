-- A qualified operator as an option's value is `OPERATOR(schema.op)`, not a list of names.
CREATE AGGREGATE my_min (sfunc = int4smaller, stype = int4, sortop = operator(pg_catalog.<));
