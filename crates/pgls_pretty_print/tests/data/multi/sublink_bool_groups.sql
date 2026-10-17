create policy "p" on public.permissions for select to authenticated using (
  exists (
    select 1
    from public.permission_set_permissions as rp
      inner join public.positions as p on p.uuid = rp.position_uuid
    where rp.permission_name = permissions.name
      and p.deleted_at is null
  )
);

create policy "p" on public.positions for select to authenticated using (
  deleted_at is null
  and (
    public.is_own_position(uuid)
    or (
      'positions.view' in (select public.current_position_permissions())
      and uuid in (select public.current_position_manageable())
    )
  )
);

select * from public.permissions where exists (select 1 from public.permission_set_permissions as rp where rp.permission_name = permissions.name);

select * from public.permissions where name = any (select permission_name from public.permission_set_permissions where position_uuid is not null);

select * from public.permissions where id > all (select permission_id from public.permission_set_permissions where position_uuid is not null);

select * from public.permissions where id = 1 or (name = 'positions.view' and deleted_at is null);

select * from public.permissions where public.permissions.permission_name = 'positions.view' and (deleted_at is null or archived_at is null) and verified;

select * from public.permissions where public.permissions.permission_name = 'positions.view' or (deleted_at is null and archived_at is null) or verified;
