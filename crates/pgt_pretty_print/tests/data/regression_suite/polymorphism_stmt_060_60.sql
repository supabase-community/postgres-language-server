create function polyf(anycompatiblemultirange) returns anycompatiblerange
as 'select range_merge($1);' language sql;
