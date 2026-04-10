create or replace function app.update_avatar_id()
    returns trigger
    language plpgsql
    security definer
    as $$
    declare
        v_handle app.valid_name;
        v_affected_account app.accounts := null;
    begin
        select (string_to_array(new.name, '/'::text))[1]::app.valid_name into v_handle;

        update app.accounts
        set avatar_id = new.id
        where handle = v_handle
        returning * into v_affected_account;

        if not v_affected_account is null then
            update auth.users u
            set
                "raw_user_meta_data" = u.raw_user_meta_data || jsonb_build_object(
                    'avatar_path', new.name
                )
            where u.id = v_affected_account.id;
        else
            update app.organizations
            set avatar_id = new.id
            where handle = v_handle;
        end if;

        return new;
    end;
    $$;

alter policy storage_objects_insert_policy
    on storage.objects
    with check (
        app.is_handle_maintainer(
            auth.uid(),
            (string_to_array(name, '/'::text))[1]::app.valid_name
        )
    );
