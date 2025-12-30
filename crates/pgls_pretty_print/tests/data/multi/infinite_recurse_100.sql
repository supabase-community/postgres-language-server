create function infinite_recurse() returns int as
'select infinite_recurse()' language sql;

SELECT version() ~ 'powerpc64[^,]*-linux-gnu'
       AS skip_test ;

select infinite_recurse();
