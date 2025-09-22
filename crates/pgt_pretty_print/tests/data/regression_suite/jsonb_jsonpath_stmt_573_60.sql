select jsonb '"12:34:56 +05:30"' @? '$.time_tz()';
