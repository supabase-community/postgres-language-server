-- pgls-format: layout=expanded
create policy "Users can view their own and managed positions"
on public.positions
for select
to authenticated
using (
  deleted_at is null
  and (
    public.is_own_position(uuid)
    or uuid in (select public.current_position_manageable())
  )
);

create policy "Members can upload files to their organization folder"
on storage.objects
for insert
to authenticated
with check (
  bucket_id = 'organization-files'
  and (storage.foldername(name))[1] in (select org_id::text from public.members where user_id = (select auth.uid()))
);

create policy "Users can update their own profile"
on public.profiles
for update
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id and deleted_at is null and status <> 'banned');

alter policy "Users can update their own profile"
on public.profiles
using ((select auth.uid()) = user_id and deleted_at is null and status <> 'banned' and verified)
with check ((select auth.uid()) = user_id);

create policy "Users can view their own profile" on public.profiles for select to authenticated using ((select auth.uid()) = user_id);
