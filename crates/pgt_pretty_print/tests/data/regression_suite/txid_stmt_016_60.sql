select txid_current() >= txid_snapshot_xmin(txid_current_snapshot());
