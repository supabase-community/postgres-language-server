-- For email provider the provider_id should be the lowercase email
-- which is availble in the email column
-- This migration was necessecitated by a recent change in identities
-- table schema by gotrue:
-- https://github.com/supabase/gotrue/blob/master/migrations/20231117164230_add_id_pkey_identities.up.sql
update auth.identities
set provider_id = email
where provider = 'email';
