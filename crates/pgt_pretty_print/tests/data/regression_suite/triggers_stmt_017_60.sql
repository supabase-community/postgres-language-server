create trigger trigger_zed
	before insert or update on trigtest
	for each row execute procedure f1_times_10();
