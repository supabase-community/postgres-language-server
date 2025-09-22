create trigger trigger_alpha
	before insert or update on trigtest
	for each row execute procedure f1_times_10();
