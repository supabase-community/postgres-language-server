-- expect_lint/safety/banAttachPartition
alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');