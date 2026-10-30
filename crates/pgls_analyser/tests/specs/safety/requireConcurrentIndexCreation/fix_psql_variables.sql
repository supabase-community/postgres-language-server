-- Placeholders are parsed as parameters, also when checking that the fix keeps the statement valid.
-- expect_lint/requireConcurrentIndexCreation
create index users_tenant_idx on users (email) where tenant = :tenant and region = :'region' and plan = $plan;
