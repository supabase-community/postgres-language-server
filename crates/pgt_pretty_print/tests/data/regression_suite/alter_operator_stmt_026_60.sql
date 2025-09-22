CREATE FUNCTION alter_op_test_fn_real_bool(real, boolean)
RETURNS boolean AS $$ SELECT NULL::BOOLEAN; $$ LANGUAGE sql IMMUTABLE;
