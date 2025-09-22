create rule r1 as on delete to t1 do delete from t2 where t2.b = old.a;
