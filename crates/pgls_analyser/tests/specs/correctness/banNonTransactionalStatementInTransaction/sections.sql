-- section: up
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum users;

-- section: up transaction:false queries:separate
vacuum users;
vacuum posts;

-- dbmate sends the section as one query, which runs in a transaction block.
-- section: up transaction:false
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum users;
-- expect_lint/banNonTransactionalStatementInTransaction
vacuum posts;

-- section: down transaction:false
vacuum users;
