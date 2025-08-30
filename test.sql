-- create schema private;

create table if not exists private.something (
  id serial primary key,
  arr double precision[]
);

create or replace function private.head(
  arr double precision[]
) returns double precision as $$
begin 
  if cardinality(arr) = 0 then 
    raise exception 'Empty array!';
  else 
    return arr[0];
  end if;
end;
$$ language plpgsql;


select head (arr) from private.something;