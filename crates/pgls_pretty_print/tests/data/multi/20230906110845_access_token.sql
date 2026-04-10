create table app.access_tokens(
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references auth.users(id) on delete cascade,
    token_hash bytea not null,
    token_name text not null check (length(token_name) <= 64),
    plaintext_suffix text not null check (length(plaintext_suffix) = 4),
    created_at timestamptz not null default now()
);

grant insert
    (id, user_id, token_hash, token_name, plaintext_suffix)
    on app.access_tokens
    to authenticated;

grant delete
    on app.access_tokens
    to authenticated;

alter table app.access_tokens enable row level security;

create policy access_tokens_select_policy
    on app.access_tokens
    as permissive
    for select
    to public
    using ( auth.uid() = user_id );

create policy access_tokens_insert_policy
    on app.access_tokens
    as permissive
    for insert
    to authenticated
    with check ( auth.uid() = user_id );

create policy access_tokens_delete_policy
    on app.access_tokens
    as permissive
    for delete
    to authenticated
    using ( auth.uid() = user_id );

create or replace function app.base64url_encode(input bytea)
    returns text
    language plpgsql
    strict
as $$
begin
    return replace(replace(encode(input, 'base64'), '/', '_'), '+', '-');
end;
$$;

create or replace function app.base64url_decode(input text)
    returns text
    language plpgsql
    strict
as $$
begin
    return decode(replace(replace(input, '-', '+'), '_', '/'), 'base64');
end;
$$;

create or replace function public.new_access_token(
    token_name text
)
    returns text
    language plpgsql
    strict
as $$
<<fn>>
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
    -- Why 21 random bytes? We are shooting for 128 bit (16 bytes) entropy. But we
    -- also show three bytes as plaintext. That takes us to a total 19 bytes.
    -- We add two bytes to make sure that the base64 encoded bytes don't have any
    -- padding, which makes it a little bit nicer to look at. That makes 21.
    token bytea = gen_random_bytes(21);
    token_hash bytea = sha256(token);
    -- Total length of the base64 encoded token is 21 * 8 / 6 = 28
    token_text text = app.base64url_encode(token);
    token_id uuid;
    -- Last 4 base64 encoded character are shown in the suffix
    plaintext_suffix text = substring(token_text from 25);
begin
    insert into app.access_tokens(user_id, token_hash, token_name, plaintext_suffix)
    values (account.id, token_hash, token_name, fn.plaintext_suffix) returning id into token_id;

    -- String returned has a length 64
    return 'dbd_' || replace(token_id::text, '-', '') || token_text;
end;
$$;

create type app.access_token_struct as (
    id uuid,
    token_name text,
    masked_token text,
    created_at timestamptz
);

create or replace function public.get_access_tokens()
    returns setof app.access_token_struct
    language plpgsql
    strict
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
begin
    return query
    select id, token_name,
            'dbd_' ||
            substring(at.id::text from 1 for 4) ||
            repeat('•', 52) ||
            at.plaintext_suffix as masked_token,
        created_at
    from app.access_tokens at
    where at.user_id = account.id;
end;
$$;

create or replace function public.delete_access_token(
    token_id uuid
)
    returns void
    language plpgsql
    strict
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
begin
    delete from app.access_tokens at
    where at.user_id = account.id and at.id = token_id;
end;
$$;

create type app.user_id_and_token_hash as (
    user_id uuid,
    token_hash bytea
);

create or replace function public.redeem_access_token(
    access_token text
)
    returns text
    language plpgsql
    security definer
    strict
as $$
declare
    token_id uuid;
    token bytea;
    tokens_row app.user_id_and_token_hash;
    token_valid boolean;
    now timestamp;
    one_hour_from_now timestamp;
    issued_at int;
    expiry_at int;
    jwt_secret text;
begin
    -- validate access token
    if length(access_token) != 64 then
        raise exception 'Invalid token';
    end if;

    if substring(access_token from 1 for 4) != 'dbd_' then
        raise exception 'Invalid token';
    end if;

    token_id := substring(access_token from 5 for 32)::uuid;
    token := app.base64url_decode(substring(access_token from 37));

    select t.user_id, t.token_hash
    into tokens_row
    from app.access_tokens t
    where t.id = token_id;

    -- TODO: do a constant time comparison
    if tokens_row.token_hash != sha256(token) then
        raise exception 'Invalid token';
    end if;

    -- Generate JWT token
    now := current_timestamp;
    one_hour_from_now := now + interval '1 hour';
    issued_at := date_part('epoch', now);
    expiry_at := date_part('epoch', one_hour_from_now);
    jwt_secret := current_setting('app.settings.jwt_secret', true);

    return sign(json_build_object(
        'aud', 'authenticated',
        'role', 'authenticated',
        'iss', 'database.dev',
        'sub', tokens_row.user_id,
        'iat', issued_at,
        'exp', expiry_at
    ), jwt_secret);
end;
$$;
