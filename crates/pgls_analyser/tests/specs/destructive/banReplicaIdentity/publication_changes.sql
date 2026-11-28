-- expect_lint/banReplicaIdentity
drop publication p;
-- expect_lint/banReplicaIdentity
alter publication p drop table t;
-- expect_lint/banReplicaIdentity
alter publication p set table t;
-- expect_lint/banReplicaIdentity
alter publication p set (publish = 'insert');
-- expect_lint/banReplicaIdentity
alter publication p set tables in schema public;
