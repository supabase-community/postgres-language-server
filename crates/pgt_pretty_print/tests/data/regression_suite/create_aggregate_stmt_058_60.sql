CREATE AGGREGATE case_agg(float8)
(
	"Stype" = internal,
	"Sfunc" = ordered_set_transition,
	"Finalfunc" = percentile_disc_final,
	"Finalfunc_extra" = true,
	"Finalfunc_modify" = read_write,
	"Parallel" = safe
);
