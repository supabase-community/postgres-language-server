SELECT oid AS datoid FROM pg_database WHERE datname = current_database() ;

BEGIN;

SELECT
	pg_advisory_xact_lock(1), pg_advisory_xact_lock_shared(2),
	pg_advisory_xact_lock(1, 1), pg_advisory_xact_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT pg_advisory_unlock_all();

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

SELECT
	pg_advisory_unlock(1), pg_advisory_unlock_shared(2),
	pg_advisory_unlock(1, 1), pg_advisory_unlock_shared(2, 2);

COMMIT;

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

BEGIN;

SELECT
	pg_advisory_xact_lock(1), pg_advisory_xact_lock_shared(2),
	pg_advisory_xact_lock(1, 1), pg_advisory_xact_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT
	pg_advisory_lock(1), pg_advisory_lock_shared(2),
	pg_advisory_lock(1, 1), pg_advisory_lock_shared(2, 2);

ROLLBACK;

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT
	pg_advisory_unlock(1), pg_advisory_unlock(1),
	pg_advisory_unlock_shared(2), pg_advisory_unlock_shared(2),
	pg_advisory_unlock(1, 1), pg_advisory_unlock(1, 1),
	pg_advisory_unlock_shared(2, 2), pg_advisory_unlock_shared(2, 2);

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

BEGIN;

SELECT
	pg_advisory_lock(1), pg_advisory_lock_shared(2),
	pg_advisory_lock(1, 1), pg_advisory_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT
	pg_advisory_xact_lock(1), pg_advisory_xact_lock_shared(2),
	pg_advisory_xact_lock(1, 1), pg_advisory_xact_lock_shared(2, 2);

ROLLBACK;

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT pg_advisory_unlock_all();

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

BEGIN;

SELECT
	pg_advisory_xact_lock(1), pg_advisory_xact_lock(1),
	pg_advisory_xact_lock_shared(2), pg_advisory_xact_lock_shared(2),
	pg_advisory_xact_lock(1, 1), pg_advisory_xact_lock(1, 1),
	pg_advisory_xact_lock_shared(2, 2), pg_advisory_xact_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

COMMIT;

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

SELECT
	pg_advisory_lock(1), pg_advisory_lock(1),
	pg_advisory_lock_shared(2), pg_advisory_lock_shared(2),
	pg_advisory_lock(1, 1), pg_advisory_lock(1, 1),
	pg_advisory_lock_shared(2, 2), pg_advisory_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT
	pg_advisory_unlock(1), pg_advisory_unlock(1),
	pg_advisory_unlock_shared(2), pg_advisory_unlock_shared(2),
	pg_advisory_unlock(1, 1), pg_advisory_unlock(1, 1),
	pg_advisory_unlock_shared(2, 2), pg_advisory_unlock_shared(2, 2);

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';

SELECT
	pg_advisory_lock(1), pg_advisory_lock(1),
	pg_advisory_lock_shared(2), pg_advisory_lock_shared(2),
	pg_advisory_lock(1, 1), pg_advisory_lock(1, 1),
	pg_advisory_lock_shared(2, 2), pg_advisory_lock_shared(2, 2);

SELECT locktype, classid, objid, objsubid, mode, granted
	FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid'
	ORDER BY classid, objid, objsubid;

SELECT pg_advisory_unlock_all();

SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND database = 'datoid';
