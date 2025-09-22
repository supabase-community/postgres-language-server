CREATE OPERATOR === (
    leftarg = integer,
    rightarg = integer,
    procedure = int4eq,
    commutator = =
);
