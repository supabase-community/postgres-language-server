SELECT lhst.i lhs,
    rhst.i rhs,
    lhst.i < rhst.i AS lt,
    lhst.i <= rhst.i AS le,
    lhst.i = rhst.i AS eq,
    lhst.i > rhst.i AS gt,
    lhst.i >= rhst.i AS ge,
    lhst.i <> rhst.i AS ne
    FROM INFINITE_INTERVAL_TBL lhst CROSS JOIN INFINITE_INTERVAL_TBL rhst
    WHERE NOT isfinite(lhst.i);
