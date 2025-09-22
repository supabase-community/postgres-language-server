CREATE OPERATOR #*# (
   leftarg = SETOF int8,
   procedure = factorial
);
