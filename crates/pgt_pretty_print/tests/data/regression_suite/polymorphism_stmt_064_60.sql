create function polyf(anycompatiblemultirange) returns anycompatible
as 'select lower($1);' language sql;
