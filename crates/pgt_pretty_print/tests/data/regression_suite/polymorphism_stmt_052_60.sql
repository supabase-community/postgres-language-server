create function polyf(anycompatiblerange) returns anycompatiblemultirange
as 'select multirange($1);' language sql;
