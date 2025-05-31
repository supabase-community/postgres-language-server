do $$
begin

begin
  create role owner superuser createdb login bypassrls;
exception 
  when duplicate_object then
   null;
  when unique_violation then
    null;
end;

begin
  create role test_login login;
exception 
  when duplicate_object then
    null;
  when unique_violation then
    null;
end;

begin
  create role test_nologin;
exception 
  when duplicate_object then
    null;
  when unique_violation then
    null;
end;

end 
$$;