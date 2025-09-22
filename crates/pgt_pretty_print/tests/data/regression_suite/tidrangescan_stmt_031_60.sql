SELECT ctid FROM tidrangescan WHERE ctid >= (SELECT NULL::tid);
