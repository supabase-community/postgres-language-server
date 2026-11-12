-- section: up transaction:false queries:separate
BEGIN;
-- expect_lint/transactionNesting
BEGIN;
-- One COMMIT ends the transaction block, also after a second BEGIN or a savepoint.
COMMIT;
-- expect_lint/transactionNesting
COMMIT;
BEGIN;
SAVEPOINT a;
COMMIT;
-- expect_lint/transactionNesting
ROLLBACK;
