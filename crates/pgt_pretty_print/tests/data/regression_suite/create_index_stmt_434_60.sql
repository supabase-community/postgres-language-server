SELECT thousand, tenthous FROM tenk1
WHERE thousand > -1 AND tenthous IN (1001,3000)
ORDER BY thousand limit 2;
