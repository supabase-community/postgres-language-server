SELECT ctid, aggfnoid::oid
FROM pg_aggregate as a
WHERE aggmtranstype != 0 AND
    (aggmtransfn = 0 OR aggminvtransfn = 0);
