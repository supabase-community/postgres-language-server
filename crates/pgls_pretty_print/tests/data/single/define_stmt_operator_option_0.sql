-- An operator as an option's value is an operator; quoted, `<` would be a type name.
CREATE AGGREGATE my_min (sfunc = int4smaller, stype = int4, sortop = <);
