create function pg_temp.mysin(float8) returns float8 language plpgsql
as 'begin return sin($1); end';
