-- A reserved keyword as an operator's argument is a string, the same as a string constant;
-- quoted as an identifier, it would be a type name.
CREATE OPERATOR @+@ ( leftarg =union, rightarg = int4, procedure = int4pl );
