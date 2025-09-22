select count(*) from tenk1
group by twenty, parallel_safe_volatile(two);
