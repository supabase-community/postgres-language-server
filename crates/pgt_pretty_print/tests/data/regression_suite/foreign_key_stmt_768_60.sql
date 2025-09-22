CREATE TABLE part3_self_fk (	-- a partitioned partition
	id bigint NOT NULL PRIMARY KEY,
	id_abc bigint
) PARTITION BY RANGE (id);
