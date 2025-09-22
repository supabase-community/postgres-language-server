create aggregate sum2(int8,int8) (
   sfunc = sum3, stype = int8,
   initcond = '0'
);
