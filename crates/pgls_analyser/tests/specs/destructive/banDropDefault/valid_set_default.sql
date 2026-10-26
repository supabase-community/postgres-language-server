-- expect_no_diagnostics
alter table users alter column created_at set default now();
alter domain status set default 'active';
