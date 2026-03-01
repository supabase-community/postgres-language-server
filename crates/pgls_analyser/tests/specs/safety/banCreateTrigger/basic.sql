-- expect_lint/safety/banCreateTrigger
create trigger my_trigger after insert on my_table for each row execute function my_func();