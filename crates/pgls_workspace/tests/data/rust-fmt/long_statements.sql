select id, name, email, created_at, updated_at, deleted_at from {schema}.users where tenant_id = {tenant} and status = {} and created_at > {} order by created_at desc limit {limit};
