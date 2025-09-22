SELECT *
FROM pg_depend as d1
WHERE refclassid = 0 OR refobjid = 0 OR
      classid = 0 OR objid = 0 OR
      deptype NOT IN ('a', 'e', 'i', 'n', 'x', 'P', 'S');
