-- expect_lint/banReplaceViewFunction
create or replace trigger tr before insert on t for each row execute function f();
