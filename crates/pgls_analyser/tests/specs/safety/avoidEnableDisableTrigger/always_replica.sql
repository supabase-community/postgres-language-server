-- expect_lint/avoidEnableDisableTrigger
alter table my_table enable always trigger my_trigger;
-- expect_lint/avoidEnableDisableTrigger
alter table my_table enable replica trigger my_trigger;
