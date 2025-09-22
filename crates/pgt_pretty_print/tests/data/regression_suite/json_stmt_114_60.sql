select (test_json->>'field3') is null as expect_true
from test_json
where json_type = 'object';
