create function polyf(anymultirange) returns anyrange
as 'select range_merge($1);' language sql;
