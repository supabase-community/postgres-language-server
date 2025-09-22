SELECT ctid, aggfnoid::oid
FROM pg_aggregate as a
WHERE aggfnoid = 0 OR aggtransfn = 0 OR
    aggkind NOT IN ('n', 'o', 'h') OR
    aggnumdirectargs < 0 OR
    (aggkind = 'n' AND aggnumdirectargs > 0) OR
    aggfinalmodify NOT IN ('r', 's', 'w') OR
    aggmfinalmodify NOT IN ('r', 's', 'w') OR
    aggtranstype = 0 OR aggtransspace < 0 OR aggmtransspace < 0;
