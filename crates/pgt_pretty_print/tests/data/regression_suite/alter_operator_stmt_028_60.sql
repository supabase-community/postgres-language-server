CREATE OPERATOR ==== (
    LEFTARG = real,
    RIGHTARG = boolean,
    PROCEDURE = alter_op_test_fn_real_bool
);
