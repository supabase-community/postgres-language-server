CREATE AGGREGATE logging_agg_strict_initcond (anyelement)
(
	stype = text,
	sfunc = logging_sfunc_strict,
	mstype = text,
	msfunc = logging_msfunc_strict,
	minvfunc = logging_minvfunc_strict,
	initcond = 'I',
	minitcond = 'MI'
);
