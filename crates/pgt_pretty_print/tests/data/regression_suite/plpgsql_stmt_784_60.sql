create or replace function unreserved_test() returns int as $$
declare
  comment int := 21;
begin
  comment := comment * 2;
  comment on function unreserved_test() is 'this is a test';
  return comment;
end
$$ language plpgsql;
