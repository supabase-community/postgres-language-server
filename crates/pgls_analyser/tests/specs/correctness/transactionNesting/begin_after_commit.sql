-- expect_lint/transactionNesting
BEGIN;
SELECT 1;
-- expect_lint/transactionNesting
COMMIT;
-- The first transaction ended, so this one isn't nested.
-- expect_lint/transactionNesting
BEGIN;
SELECT 2;
-- expect_lint/transactionNesting
COMMIT;
