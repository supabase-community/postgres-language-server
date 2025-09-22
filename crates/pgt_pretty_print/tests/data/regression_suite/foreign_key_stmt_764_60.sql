CREATE TABLE parted_self_fk (
    id bigint NOT NULL PRIMARY KEY,
    id_abc bigint,
    FOREIGN KEY (id_abc) REFERENCES parted_self_fk(id)
)
PARTITION BY RANGE (id);
