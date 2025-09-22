SELECT indexrelid, indrelid
FROM pg_index
WHERE indexrelid = 0 OR indrelid = 0 OR
      indnatts <= 0 OR indnatts > 32;
