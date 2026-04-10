update app.packages
-- Fix typo in spelling of "packages"
set control_description = 'Install packages from the database.dev registry'
where handle = 'supabase' and partial_name = 'dbdev';
