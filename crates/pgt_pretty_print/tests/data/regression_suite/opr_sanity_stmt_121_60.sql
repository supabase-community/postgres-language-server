SELECT a1.amprocfamily, a1.amprocnum
FROM pg_amproc as a1
WHERE a1.amprocfamily = 0 OR a1.amproclefttype = 0 OR a1.amprocrighttype = 0
    OR a1.amprocnum < 0 OR a1.amproc = 0;
