-- expect_lint/safety/transactionNesting
BEGIN;
SELECT 1;
-- expect_lint/safety/transactionNesting
COMMIT;
