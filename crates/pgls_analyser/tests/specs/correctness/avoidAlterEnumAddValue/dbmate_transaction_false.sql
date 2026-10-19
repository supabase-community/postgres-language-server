-- sectionMarkers: dbmate
-- migrate:up transaction:false
ALTER TYPE status ADD VALUE 'archived';

-- migrate:down
-- expect_lint/avoidAlterEnumAddValue
ALTER TYPE status ADD VALUE 'deleted';
