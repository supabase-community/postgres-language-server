select setting as segsize
from pg_settings where name = 'wal_segment_size'
