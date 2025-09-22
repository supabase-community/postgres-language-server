create trigger trigtest_a_row_tg after insert or update or delete on trigtest
for each row execute procedure trigtest();
