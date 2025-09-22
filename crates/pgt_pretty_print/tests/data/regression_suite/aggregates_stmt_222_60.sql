select t1.f1 from t1 left join t2 using (f1) group by f1;
