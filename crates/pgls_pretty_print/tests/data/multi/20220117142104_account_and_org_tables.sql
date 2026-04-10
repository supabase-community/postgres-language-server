insert into storage.buckets ("id", "name")
values ('avatars', 'avatars');

create table app.handle_registry(
    /*
    Enforces uniqueness of handles across orgs and accounts
    e.g. jsmith or supabase
    */
    handle app.valid_name primary key not null,
    is_organization boolean not null,
    created_at timestamptz not null default now(),
    unique (handle, is_organization)
);

create table app.accounts(
    -- 1:1 with auth.users
    id uuid primary key references auth.users(id),
    handle app.valid_name not null unique,
    is_organization boolean generated always as (false) stored,
    avatar_id uuid references storage.objects(id),
    display_name text check (length(display_name) <= 128),
    bio text check (length(bio) <= 512),
    contact_email app.email_address,
    created_at timestamptz not null default now(),

    constraint fk_handle_registry
        foreign key (handle, is_organization)
        references app.handle_registry(handle, is_organization)
);

create or replace function app.register_account()
    returns trigger
    language plpgsql
    security definer
    as $$
    begin
        insert into app.handle_registry (handle, is_organization)
          values (
            new.raw_user_meta_data ->> 'handle',
            false
          );

        insert into app.accounts (id, handle, display_name, bio, contact_email)
          values (
            new.id,
            new.raw_user_meta_data ->> 'handle',
            new.raw_user_meta_data ->> 'display_name',
            new.raw_user_meta_data ->> 'bio',
            new.raw_user_meta_data ->> 'contact_email'
          );
          return new;
    end;
    $$;

create or replace trigger on_auth_user_created
    after insert on auth.users
    for each row execute procedure app.register_account();

create table app.organizations(
    id uuid primary key default gen_random_uuid(),
    handle app.valid_name not null unique,
    is_organization boolean generated always as (true) stored,
    avatar_id uuid references storage.objects(id),
    display_name text check (length(display_name) <= 128),
    bio text check (length(bio) <= 512),
    contact_email app.email_address,
    -- enforced so organization always have at least 1 admin member
    created_at timestamptz not null default now(),

    constraint fk_handle_registry
        foreign key (handle, is_organization)
        references app.handle_registry(handle, is_organization)
);

create type app.membership_role as enum ('maintainer');

create table app.members(
    id uuid primary key default uuid_generate_v4(),
    organization_id uuid not null references app.organizations(id),
    account_id uuid not null references app.accounts(id),
    role app.membership_role not null,
    created_at timestamptz not null default now(),
    unique (organization_id, account_id)
);

create or replace function app.register_organization_creator_as_member()
    returns trigger
    language plpgsql
    security definer
    as $$
    begin
        insert into app.members(organization_id, account_id, role)
        values (new.id, auth.uid(), 'maintainer');

        return new;
    end;
    $$;

create or replace trigger on_app_organization_created
    after insert on app.organizations
    for each row execute procedure app.register_organization_creator_as_member();

create or replace function app.update_avatar_id()
    returns trigger
    language plpgsql
    security definer
    as $$
    declare
        v_handle app.valid_name;
        v_affected_account app.accounts := null;
    begin
        select (string_to_array(new.name, '-'::text))[1]::app.valid_name into v_handle;

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

create or replace trigger on_storage_object_created
    after insert on storage.objects
    for each row when(new.bucket_id = 'avatars') execute procedure app.update_avatar_id();
