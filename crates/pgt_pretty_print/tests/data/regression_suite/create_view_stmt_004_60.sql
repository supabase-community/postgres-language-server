SELECT *
   INTO TABLE ramp
   FROM ONLY road
   WHERE name ~ '.*Ramp';
