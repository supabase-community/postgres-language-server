-- A temporal primary key (Postgres 18) needs a GiST index, so USING INDEX is no alternative
-- expect_lint/addingPrimaryKeyConstraint
ALTER TABLE bookings ADD CONSTRAINT bookings_pkey PRIMARY KEY (room_id, during WITHOUT OVERLAPS);
