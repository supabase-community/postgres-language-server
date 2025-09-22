select jsonb '[1]' @? 'strict $[10000000000000000]';
