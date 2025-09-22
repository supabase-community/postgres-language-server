create function makedcomp(r float8, i float8) returns dcomptype
as 'select row(r, i)' language sql;
