SELECT *
FROM pg_cast c
WHERE castsource = casttarget AND castfunc = 0;
