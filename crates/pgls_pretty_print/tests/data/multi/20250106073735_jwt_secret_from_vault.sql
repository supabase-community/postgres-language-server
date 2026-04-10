-- app.settings.jwt_secret has been removed, see https://github.com/orgs/supabase/discussions/30606
-- now we fetch the secret from vault. The redeem_access_token function is the same as before
-- (see file 20230906110845_access_token.sql) except the part where we fetch the jwt_secret.
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

    -- read the jwt secret from vault
    select decrypted_secret
    into jwt_secret
    from vault.decrypted_secrets
    where name = 'app.jwt_secret';

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
