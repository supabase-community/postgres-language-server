create function add_one(value int8, step int8 default 1) returns int8 language sql as 'select value + step';
select add_one(1);
select add_one(1, 2);
-- expect_lint/unknownFunction
select missing_function();
-- expect_lint/unknownFunction
select add_one(1, 2, 3);
