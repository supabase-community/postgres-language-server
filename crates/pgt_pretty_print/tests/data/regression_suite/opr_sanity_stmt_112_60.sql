SELECT a1.amopfamily, a1.amopstrategy
FROM pg_amop as a1
WHERE a1.amopfamily = 0 OR a1.amoplefttype = 0 OR a1.amoprighttype = 0
    OR a1.amopopr = 0 OR a1.amopmethod = 0 OR a1.amopstrategy < 1;
