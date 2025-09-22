CREATE OPERATOR #*# (
   rightarg = SETOF int8,
   procedure = factorial
);
