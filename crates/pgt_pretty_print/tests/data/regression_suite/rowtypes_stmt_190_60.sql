create function longname(fullname) returns text language sql
as $$select $1.first || ' ' || $1.last$$;
