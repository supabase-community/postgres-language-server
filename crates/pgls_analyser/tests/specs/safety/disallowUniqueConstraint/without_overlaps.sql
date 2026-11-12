-- A temporal unique constraint (Postgres 18) needs a GiST index, so USING INDEX is no alternative
-- expect_lint/disallowUniqueConstraint
ALTER TABLE bookings ADD CONSTRAINT bookings_room_key UNIQUE (room_id, during WITHOUT OVERLAPS);
