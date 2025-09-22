CREATE TABLE gtest24r (a int PRIMARY KEY, b gtestdomain1range GENERATED ALWAYS AS (gtestdomain1range(a, a + 5)) STORED);
