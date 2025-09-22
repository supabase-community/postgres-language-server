SELECT reltoastrelid::regclass AS reltoastname FROM pg_class
  WHERE oid = 'toasttest'::regclass ;
