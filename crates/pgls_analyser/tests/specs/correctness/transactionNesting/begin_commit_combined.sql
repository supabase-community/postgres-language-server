-- expect_lint/transactionNesting
BEGIN;
SELECT 1;
-- expect_lint/transactionNesting
COMMIT;
