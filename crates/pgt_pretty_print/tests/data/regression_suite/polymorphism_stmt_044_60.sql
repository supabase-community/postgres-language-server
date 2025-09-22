create function polyf(anyrange) returns anymultirange
as 'select multirange($1);' language sql;
