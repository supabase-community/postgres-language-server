create constraint trigger whoami after insert on defer_trig
  deferrable initially deferred
  for each row
  execute function whoami();
