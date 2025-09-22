select count(distinct utc_offset) >= 24 as ok from pg_timezone_abbrevs;
