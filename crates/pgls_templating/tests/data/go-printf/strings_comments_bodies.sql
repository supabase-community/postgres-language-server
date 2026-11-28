select '%s', E'%d', $$ select %v $$; -- %q
/* %x */
create function f() returns int language sql as $body$ select %d $body$;
