create function f() returns int language sql as $fn$ select 1 /* myfn$ */ $fn$;

create function g() returns text language plpgsql as $body$
begin
  -- not the end: xbody$ $other$ body$
  return 'a; b';
end;
$body$;

select 2;
