select * from partitioned where row(a,b)::partitioned = '(1,2)'::partitioned;
