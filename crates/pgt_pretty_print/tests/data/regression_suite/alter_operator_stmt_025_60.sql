CREATE FUNCTION alter_op_test_fn_bool_real(boolean, real)
RETURNS boolean AS $$ SELECT NULL::BOOLEAN; $$ LANGUAGE sql IMMUTABLE;
