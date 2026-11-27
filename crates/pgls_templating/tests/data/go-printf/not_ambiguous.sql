select a as %s, %s.%s, app_%s from t where id = %d and x in (%s);
select * from %s join %s on %s.id = %d limit %d offset %d;
select 7 %% %d, a::%s, '%s' as x, "%s" from t; -- a %s
select a "100%%" from t;
