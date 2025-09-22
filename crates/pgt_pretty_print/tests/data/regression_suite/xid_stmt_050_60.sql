select  pg_snapshot_xmin(snap),
	pg_snapshot_xmax(snap),
	pg_snapshot_xip(snap)
from snapshot_test order by nr;
