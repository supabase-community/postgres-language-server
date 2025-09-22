create function first_el_transfn(anyarray, anyelement) returns anyarray as
'select $1 || $2' language sql immutable;
