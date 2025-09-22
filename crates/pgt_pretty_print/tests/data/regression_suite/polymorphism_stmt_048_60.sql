create function polyf(anymultirange) returns anyelement
as 'select lower($1);' language sql;
