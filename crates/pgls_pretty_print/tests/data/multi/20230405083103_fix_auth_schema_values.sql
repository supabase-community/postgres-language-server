update auth.users
set
  created_at = now(),
  updated_at = now(),
  email_confirmed_at = now(),
  confirmation_token = '',
  recovery_token = '',
  email_change_token_new = '',
  email_change = '';

insert into
  auth.identities (
    id,
    provider_id,
    provider,
    user_id,
    identity_data,
    created_at,
    updated_at
  )
select
  gen_random_uuid(),
  email,
  'email' as provider,
  id as user_id,
  jsonb_build_object('sub', id, 'email', email) as identity_data,
  created_at,
  updated_at
from
  auth.users;
