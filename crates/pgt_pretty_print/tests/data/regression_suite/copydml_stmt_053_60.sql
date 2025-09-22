create trigger qqqaf after insert or update or delete on copydml_test
    for each row execute procedure qqq_trig();
