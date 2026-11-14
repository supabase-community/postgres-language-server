select * from bookings b where (b.starts_at, b.ends_at) overlaps (date '2020-01-01', date '2020-02-01') and b.id > 1;
