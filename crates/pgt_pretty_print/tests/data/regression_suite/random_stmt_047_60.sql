SELECT n, random(0, trim_scale(abs(1 - 10.0^(-n)))) FROM generate_series(-20, 20) n;
