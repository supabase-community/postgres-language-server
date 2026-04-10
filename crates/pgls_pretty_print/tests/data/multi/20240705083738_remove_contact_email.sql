drop view if exists public.accounts;

create view
  public.accounts
with
  (security_invoker = true) as
select
  acc.id,
  acc.handle,
  obj.name as avatar_path,
  acc.display_name,
  acc.bio,
  acc.created_at
from
  app.accounts acc
  left join storage.objects obj on acc.avatar_id = obj.id;

drop view if exists public.organizations;

create view
  public.organizations
with
  (security_invoker = true) as
select
  org.id,
  org.handle,
  obj.name as avatar_path,
  org.display_name,
  org.bio,
  org.created_at
from
  app.organizations org
  left join storage.objects obj on org.avatar_id = obj.id;
