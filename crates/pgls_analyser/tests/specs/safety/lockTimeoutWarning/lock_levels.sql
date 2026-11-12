-- VALIDATE CONSTRAINT, SET STATISTICS and the autovacuum storage parameters take SHARE UPDATE
-- EXCLUSIVE, which doesn't block reads or writes.
alter table users validate constraint users_email_check;
alter table users set (autovacuum_enabled = false);
alter table users alter column email set statistics 500;
-- Triggers and foreign keys take SHARE ROW EXCLUSIVE, which blocks writes.
-- expect_lint/lockTimeoutWarning
alter table users enable trigger users_audit;
-- expect_lint/lockTimeoutWarning
alter table users add constraint users_org_fk foreign key (org_id) references orgs (id) not valid;
-- ATTACH PARTITION locks the partition ACCESS EXCLUSIVE.
-- expect_lint/lockTimeoutWarning
alter table events attach partition events_2024 for values from ('2024-01-01') to ('2025-01-01');
