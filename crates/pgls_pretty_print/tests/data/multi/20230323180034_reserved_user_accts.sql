insert into auth.users(instance_id, id, aud, role, email, encrypted_password, email_confirmed_at, raw_app_meta_data, raw_user_meta_data, is_sso_user)
values
    (
        '00000000-0000-0000-0000-000000000000', gen_random_uuid(), 'authenticated', 'authenticated', 'oliver@oliverrice.com', 'TBD', now(),
        '{"provider": "email", "providers": ["email"]}', '{"handle": "olirice", "display_name": "Oli", "bio": "Supabase Staff"}', false
    ),
    (
        '00000000-0000-0000-0000-000000000000', gen_random_uuid(), 'authenticated', 'authenticated', 'alaister@supabase.io', 'TBD', now(),
        '{"provider": "email", "providers": ["email"]}', '{"handle": "alaister", "display_name": "Alaister", "bio": "Supabase Staff"}', false
    ),
    (
        '00000000-0000-0000-0000-000000000000', gen_random_uuid(), 'authenticated', 'authenticated', 'copple@supabase.io', 'TBD', now(),
        '{"provider": "email", "providers": ["email"]}', '{"handle": "kiwicopple", "display_name": "Copple", "bio": "Supabase Staff"}', false
    ),
    (
        '00000000-0000-0000-0000-000000000000', gen_random_uuid(), 'authenticated', 'authenticated', 'michel@supabase.io', 'TBD', now(),
        '{"provider": "email", "providers": ["email"]}', '{"handle": "michelp", "display_name": "Michele", "bio": "Supabase Staff"}', false
    ),
    (
        '00000000-0000-0000-0000-000000000000', gen_random_uuid(), 'authenticated', 'authenticated', 'mark@supabase.io', 'TBD', now(),
        '{"provider": "email", "providers": ["email"]}', '{"handle": "burggraf", "display_name": "Mark", "bio": "Supabase Staff"}', false
    );

insert into app.handle_registry(handle, is_organization)
values
    ('supabase', true),
    ('langchain', true),
    -- Reserve common impersonation handles
    ('admin', false),
    ('administrator', false),
    ('superuser', false),
    ('superadmin', false),
    ('root', false),
    ('user', false),
    ('guest', false),
    ('anon', false),
    ('authenticated', false),
    ('sysadmin', false),
    ('support', false),
    ('manager', false),
    ('default', false),
    ('staff', false),
    ('help', false),
    ('helpdesk', false),
    ('test', false),
    ('password', false),
    ('demo', false),
    ('service', false),
    ('info', false),
    ('webmaster', false),
    ('security', false),
    ('installer', false);

begin;
    -- Required for trigger on handle registry
    select app.simulate_login('oliver@oliverrice.com');

    insert into app.organizations(handle, display_name, bio)
    values
        ('supabase', 'Supabase', 'Build in a weekend, scale to millions');
end;

insert into app.members(organization_id, account_id, role)
select
    o.id,
    acc.id,
    'maintainer'
from
    app.organizations o,
    app.accounts acc
where
    -- olirice is already a member because that account created it
    acc.handle <> 'olirice';

begin;
    -- Required for trigger on handle registry
    select app.simulate_login('oliver@oliverrice.com');

    insert into app.organizations(handle, display_name, bio)
    values
        ('langchain', 'LangChain', 'LangChain is a framework for developing applications powered by language models');
end;
