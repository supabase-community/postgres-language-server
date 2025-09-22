SELECT a1.amprocfamily, a1.amproc, p1.prosrc
FROM pg_amproc AS a1, pg_proc AS p1
WHERE a1.amproc = p1.oid AND
    a1.amproclefttype != a1.amprocrighttype AND
    p1.provolatile = 'v';
