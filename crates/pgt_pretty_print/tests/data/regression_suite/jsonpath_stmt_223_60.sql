SELECT str as jsonpath,
       pg_input_is_valid(str,'jsonpath') as ok,
       errinfo.sql_error_code,
       errinfo.message,
       errinfo.detail,
       errinfo.hint
FROM unnest(ARRAY['$ ? (@ like_regex "pattern" flag "smixq")'::text,
                  '$ ? (@ like_regex "pattern" flag "a")',
                  '@ + 1',
                  '00',
                  '1a']) str,
     LATERAL pg_input_error_info(str, 'jsonpath') as errinfo;
