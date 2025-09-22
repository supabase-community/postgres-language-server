CREATE INDEX jidx_array ON testjsonb USING gin((j->'array'));
