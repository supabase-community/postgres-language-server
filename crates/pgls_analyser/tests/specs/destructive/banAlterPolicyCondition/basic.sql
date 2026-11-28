-- expect_lint/banAlterPolicyCondition
-- expect_lint/banAlterPolicyCondition
alter policy p on t using (true) with check (false);
