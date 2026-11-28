select   * from t, %s   where a = %d;
select * from (select * from a, %s) s,   %s.u;
