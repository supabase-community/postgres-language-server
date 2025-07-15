create table
  unknown_users (id serial primary key, address text, email text);

drop table unknown_users;

select
  *
from
  unknown_users;

sel 1;



create function test_organisation_id ()
    returns setof text
    language plpgsql
    security invoker
    as $$
    declre
        v_organisation_id uuid;
begin
    return next is(private.organisation_id(), v_organisation_id, 'should return organisation_id of token');
end
$$;

