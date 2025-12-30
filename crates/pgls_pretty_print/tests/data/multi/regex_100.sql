set standard_conforming_strings = on;

select 'bbbbb' ~ '^([bc])\1*$' as t;

select 'ccc' ~ '^([bc])\1*$' as t;

select 'xxx' ~ '^([bc])\1*$' as f;

select 'bbc' ~ '^([bc])\1*$' as f;

select 'b' ~ '^([bc])\1*$' as t;

select 'abc abc abc' ~ '^(\w+)( \1)+$' as t;

select 'abc abd abc' ~ '^(\w+)( \1)+$' as f;

select 'abc abc abd' ~ '^(\w+)( \1)+$' as f;

select 'abc abc abc' ~ '^(.+)( \1)+$' as t;

select 'abc abd abc' ~ '^(.+)( \1)+$' as f;

select 'abc abc abd' ~ '^(.+)( \1)+$' as f;

select substring('asd TO foo' from ' TO (([a-z0-9._]+|"([^"]+|"")+")+)');

select substring('a' from '((a))+');

select substring('a' from '((a)+)');

select regexp_match('abc', '');

select regexp_match('abc', 'bc');

select regexp_match('abc', 'd') is null;

select regexp_match('abc', '(B)(c)', 'i');

select regexp_match('abc', 'Bd', 'ig');

select regexp_matches('ab', 'a(?=b)b*');

select regexp_matches('a', 'a(?=b)b*');

select regexp_matches('abc', 'a(?=b)b*(?=c)c*');

select regexp_matches('ab', 'a(?=b)b*(?=c)c*');

select regexp_matches('ab', 'a(?!b)b*');

select regexp_matches('a', 'a(?!b)b*');

select regexp_matches('b', '(?=b)b');

select regexp_matches('a', '(?=b)b');

select regexp_matches('abb', '(?<=a)b*');

select regexp_matches('a', 'a(?<=a)b*');

select regexp_matches('abc', 'a(?<=a)b*(?<=b)c*');

select regexp_matches('ab', 'a(?<=a)b*(?<=b)c*');

select regexp_matches('ab', 'a*(?<!a)b*');

select regexp_matches('ab', 'a*(?<!a)b+');

select regexp_matches('b', 'a*(?<!a)b+');

select regexp_matches('a', 'a(?<!a)b*');

select regexp_matches('b', '(?<=b)b');

select regexp_matches('foobar', '(?<=f)b+');

select regexp_matches('foobar', '(?<=foo)b+');

select regexp_matches('foobar', '(?<=oo)b+');

select 'xz' ~ 'x(?=[xy])';

select 'xy' ~ 'x(?=[xy])';

select 'xz' ~ 'x(?![xy])';

select 'xy' ~ 'x(?![xy])';

select 'x'  ~ 'x(?![xy])';

select 'xyy' ~ '(?<=[xy])yy+';

select 'zyy' ~ '(?<=[xy])yy+';

select 'xyy' ~ '(?<![xy])yy+';

select 'zyy' ~ '(?<![xy])yy+';

select * from pg_proc where proname ~ 'abc';

select * from pg_proc where proname ~ '^abc';

select * from pg_proc where proname ~ '^abc$';

select * from pg_proc where proname ~ '^abcd*e';

select * from pg_proc where proname ~ '^abc+d';

select * from pg_proc where proname ~ '^(abc)(def)';

select * from pg_proc where proname ~ '^(abc)$';

select * from pg_proc where proname ~ '^(abc)?d';

select * from pg_proc where proname ~ '^abcd(x|(?=\w\w)q)';

select 'a' ~ '($|^)*';

select 'a' ~ '(^)+^';

select 'a' ~ '$($$)+';

select 'a' ~ '($^)+';

select 'a' ~ '(^$)*';

select 'aa bb cc' ~ '(^(?!aa))+';

select 'aa x' ~ '(^(?!aa)(?!bb)(?!cc))+';

select 'bb x' ~ '(^(?!aa)(?!bb)(?!cc))+';

select 'cc x' ~ '(^(?!aa)(?!bb)(?!cc))+';

select 'dd x' ~ '(^(?!aa)(?!bb)(?!cc))+';

select 'a' ~ '((((((a)*)*)*)*)*)*';

select 'a' ~ '((((((a+|)+|)+|)+|)+|)+|)';

select 'x' ~ 'abcd(\m)+xyz';

select 'a' ~ '^abcd*(((((^(a c(e?d)a+|)+|)+|)+|)+|a)+|)';

select 'x' ~ 'a^(^)bcd*xy(((((($a+|)+|)+|)+$|)+|)+|)^$';

select 'x' ~ 'xyz(\Y\Y)+';

select 'x' ~ 'x|(?:\M)+';

select 'x' ~ repeat('x*y*z*', 1000);

select 'Programmer' ~ '(\w).*?\1' as t;

select regexp_matches('Programmer', '(\w)(.*?\1)', 'g');

select regexp_matches('foo/bar/baz',
                      '^([^/]+?)(?:/([^/]+?))(?:/([^/]+?))?$', '');

select regexp_matches('llmmmfff', '^(l*)(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*){1,1}(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*){1,1}?(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*){1,1}?(.*){1,1}?(f*)$');

select regexp_matches('llmmmfff', '^(l*?)(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*?){1,1}(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*?){1,1}?(.*)(f*)$');

select regexp_matches('llmmmfff', '^(l*?){1,1}?(.*){1,1}?(f*)$');

select 'a' ~ '$()|^\1';

select 'a' ~ '.. ()|\1';

select 'a' ~ '()*\1';

select 'a' ~ '()+\1';

select 'xxx' ~ '(.){0}(\1)' as f;

select 'xxx' ~ '((.)){0}(\2)' as f;

select 'xyz' ~ '((.)){0}(\2){0}' as t;

select 'abcdef' ~ '^(.)\1|\1.' as f;

select 'abadef' ~ '^((.)\2|..)\2' as f;

select regexp_match('xy', '.|...');

select regexp_match('xyz', '.|...');

select regexp_match('xy', '.*');

select regexp_match('fooba', '(?:..)*');

select regexp_match('xyz', repeat('.', 260));

select regexp_match('foo', '(?:.|){99}');

select 'xyz' ~ 'x(\w)(?=\1)';

select 'xyz' ~ 'x(\w)(?=(\1))';

select 'a' ~ '\x7fffffff';
