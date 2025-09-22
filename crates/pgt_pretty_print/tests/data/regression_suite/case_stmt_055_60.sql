SELECT CASE make_ad(1,2)
  WHEN array[2,4]::arrdomain THEN 'wrong'
  WHEN array[2,5]::arrdomain THEN 'still wrong'
  WHEN array[1,2]::arrdomain THEN 'right'
  END;
