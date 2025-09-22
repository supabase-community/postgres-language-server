insert into tuplesest_parted select i%200, i%300, i%400 from generate_series(1, 1000)i;
