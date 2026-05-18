-- expect_lint/safety/avoidCreateTrigger
create trigger my_trigger after insert on my_table for each row execute function my_func();