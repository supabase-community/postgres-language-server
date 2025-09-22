CREATE OPERATOR === (
    LEFTARG = boolean,
    RIGHTARG = real,
    PROCEDURE = alter_op_test_fn_bool_real
);
