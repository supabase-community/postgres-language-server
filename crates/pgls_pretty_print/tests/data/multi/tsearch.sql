SELECT oid, prsname
FROM pg_ts_parser
WHERE prsnamespace = 0 OR prsstart = 0 OR prstoken = 0 OR prsend = 0 OR
      -- prsheadline is optional
      prslextype = 0;

SELECT oid, dictname
FROM pg_ts_dict
WHERE dictnamespace = 0 OR dictowner = 0 OR dicttemplate = 0;

SELECT oid, tmplname
FROM pg_ts_template
WHERE tmplnamespace = 0 OR tmpllexize = 0;

SELECT oid, cfgname
FROM pg_ts_config
WHERE cfgnamespace = 0 OR cfgowner = 0 OR cfgparser = 0;

SELECT mapcfg, maptokentype, mapseqno
FROM pg_ts_config_map
WHERE mapcfg = 0 OR mapdict = 0;

SELECT * FROM
  ( SELECT oid AS cfgid, (ts_token_type(cfgparser)).tokid AS tokid
    FROM pg_ts_config ) AS tt
RIGHT JOIN pg_ts_config_map AS m
    ON (tt.cfgid=m.mapcfg AND tt.tokid=m.maptokentype)
WHERE
    tt.cfgid IS NULL OR tt.tokid IS NULL;

CREATE TABLE test_tsvector(
	t text,
	a tsvector
);

COPY test_tsvector FROM 'filename';

ANALYZE test_tsvector;

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

create index wowidx on test_tsvector using gist (a);

SET enable_seqscan=OFF;

SET enable_indexscan=ON;

SET enable_bitmapscan=OFF;

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

SET enable_indexscan=OFF;

SET enable_bitmapscan=ON;

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(foo=1));

CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(siglen=0));

CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(siglen=2048));

CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(siglen=100,foo='bar'));

CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(siglen=100, siglen = 200));

CREATE INDEX wowidx2 ON test_tsvector USING gist (a tsvector_ops(siglen=1));

DROP INDEX wowidx;

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

DROP INDEX wowidx2;

CREATE INDEX wowidx ON test_tsvector USING gist (a tsvector_ops(siglen=484));

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

DROP INDEX wowidx;

CREATE INDEX wowidx ON test_tsvector USING gin (a);

SET enable_seqscan=OFF;

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr|qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr&qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq&yt';

SELECT count(*) FROM test_tsvector WHERE a @@ 'eq|yt';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq&yt)|(wr&qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '(eq|yt)&(wr|qh)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'w:*|q:*';

SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');

SELECT count(*) FROM test_tsvector WHERE a @@ 'no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';

SELECT count(*) FROM test_tsvector WHERE a @@ 'pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ 'qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!pl <-> !yh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!yh <-> pl';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qe <2> qt';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(pl <-> yh)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(yh <-> pl)';

SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:A';

SELECT count(*) FROM test_tsvector WHERE a @@ '!wd:D';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qh';

SELECT count(*) FROM test_tsvector WHERE a @@ '!qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr' AND a @@ '!qh';

SELECT count(*) FROM test_tsvector WHERE a @@ 'wr' AND a @@ '!qh';

RESET enable_seqscan;

INSERT INTO test_tsvector VALUES ('???', 'DFG:1A,2B,6C,10 FGH');

SELECT * FROM ts_stat('SELECT a FROM test_tsvector') ORDER BY ndoc DESC, nentry DESC, word LIMIT 10;

SELECT * FROM ts_stat('SELECT a FROM test_tsvector', 'AB') ORDER BY ndoc DESC, nentry DESC, word;

SELECT ts_lexize('english_stem', 'skies');

SELECT ts_lexize('english_stem', 'identity');

SELECT * FROM ts_token_type('default');

SELECT * FROM ts_parse('default', '345 qwe@efd.r '' http://www.com/ http://aew.werc.ewr/?ad=qwe&dw 1aew.werc.ewr/?ad=qwe&dw 2aew.werc.ewr http://3aew.werc.ewr/?ad=qwe&dw http://4aew.werc.ewr http://5aew.werc.ewr:8100/?  ad=qwe&dw 6aew.werc.ewr:8100/?ad=qwe&dw 7aew.werc.ewr:8100/?ad=qwe&dw=%20%32 +4.0e-10 qwe qwe qwqwe 234.435 455 5.005 teodor@stack.net teodor@123-stack.net 123_teodor@stack.net 123-teodor@stack.net qwe-wer asdf <fr>qwer jf sdjk<we hjwer <werrwe> ewr1> ewri2 <a href="qwe<qwe>">
/usr/local/fff /awdf/dwqe/4325 rewt/ewr wefjn /wqe-324/ewr gist.h gist.h.c gist.c. readline 4.2 4.2. 4.2, readline-4.2 readline-4.2. 234
<i <b> wow  < jqw <> qwerty');

SELECT to_tsvector('english', '345 qwe@efd.r '' http://www.com/ http://aew.werc.ewr/?ad=qwe&dw 1aew.werc.ewr/?ad=qwe&dw 2aew.werc.ewr http://3aew.werc.ewr/?ad=qwe&dw http://4aew.werc.ewr http://5aew.werc.ewr:8100/?  ad=qwe&dw 6aew.werc.ewr:8100/?ad=qwe&dw 7aew.werc.ewr:8100/?ad=qwe&dw=%20%32 +4.0e-10 qwe qwe qwqwe 234.435 455 5.005 teodor@stack.net teodor@123-stack.net 123_teodor@stack.net 123-teodor@stack.net qwe-wer asdf <fr>qwer jf sdjk<we hjwer <werrwe> ewr1> ewri2 <a href="qwe<qwe>">
/usr/local/fff /awdf/dwqe/4325 rewt/ewr wefjn /wqe-324/ewr gist.h gist.h.c gist.c. readline 4.2 4.2. 4.2, readline-4.2 readline-4.2. 234
<i <b> wow  < jqw <> qwerty');

SELECT length(to_tsvector('english', '345 qwe@efd.r '' http://www.com/ http://aew.werc.ewr/?ad=qwe&dw 1aew.werc.ewr/?ad=qwe&dw 2aew.werc.ewr http://3aew.werc.ewr/?ad=qwe&dw http://4aew.werc.ewr http://5aew.werc.ewr:8100/?  ad=qwe&dw 6aew.werc.ewr:8100/?ad=qwe&dw 7aew.werc.ewr:8100/?ad=qwe&dw=%20%32 +4.0e-10 qwe qwe qwqwe 234.435 455 5.005 teodor@stack.net teodor@123-stack.net 123_teodor@stack.net 123-teodor@stack.net qwe-wer asdf <fr>qwer jf sdjk<we hjwer <werrwe> ewr1> ewri2 <a href="qwe<qwe>">
/usr/local/fff /awdf/dwqe/4325 rewt/ewr wefjn /wqe-324/ewr gist.h gist.h.c gist.c. readline 4.2 4.2. 4.2, readline-4.2 readline-4.2. 234
<i <b> wow  < jqw <> qwerty'));

SELECT * from ts_debug('english', '<myns:foo-bar_baz.blurfl>abc&nm1;def&#xa9;ghi&#245;jkl</myns:foo-bar_baz.blurfl>');

SELECT * from ts_debug('english', 'http://www.harewoodsolutions.co.uk/press.aspx</span>');

SELECT * from ts_debug('english', 'http://aew.wer0c.ewr/id?ad=qwe&dw<span>');

SELECT * from ts_debug('english', 'http://5aew.werc.ewr:8100/?');

SELECT * from ts_debug('english', '5aew.werc.ewr:8100/?xx');

SELECT token, alias,
  dictionaries, dictionaries is null as dnull, array_dims(dictionaries) as ddims,
  lexemes, lexemes is null as lnull, array_dims(lexemes) as ldims
from ts_debug('english', 'a title');

SELECT to_tsquery('english', 'qwe & sKies ');

SELECT to_tsquery('simple', 'qwe & sKies ');

SELECT to_tsquery('english', '''the wether'':dc & ''           sKies '':BC ');

SELECT to_tsquery('english', 'asd&(and|fghj)');

SELECT to_tsquery('english', '(asd&and)|fghj');

SELECT to_tsquery('english', '(asd&!and)|fghj');

SELECT to_tsquery('english', '(the|and&(i&1))&fghj');

SELECT plainto_tsquery('english', 'the and z 1))& fghj');

SELECT plainto_tsquery('english', 'foo bar') && plainto_tsquery('english', 'asd');

SELECT plainto_tsquery('english', 'foo bar') || plainto_tsquery('english', 'asd fg');

SELECT plainto_tsquery('english', 'foo bar') || !!plainto_tsquery('english', 'asd fg');

SELECT plainto_tsquery('english', 'foo bar') && 'asd | fg';

SELECT to_tsquery('english', '!(a & !b) & c');

SELECT to_tsquery('english', '!(a & !b)');

SELECT to_tsquery('english', '(1 <-> 2) <-> a');

SELECT to_tsquery('english', '(1 <-> a) <-> 2');

SELECT to_tsquery('english', '(a <-> 1) <-> 2');

SELECT to_tsquery('english', 'a <-> (1 <-> 2)');

SELECT to_tsquery('english', '1 <-> (a <-> 2)');

SELECT to_tsquery('english', '1 <-> (2 <-> a)');

SELECT to_tsquery('english', '(1 <-> 2) <3> a');

SELECT to_tsquery('english', '(1 <-> a) <3> 2');

SELECT to_tsquery('english', '(a <-> 1) <3> 2');

SELECT to_tsquery('english', 'a <3> (1 <-> 2)');

SELECT to_tsquery('english', '1 <3> (a <-> 2)');

SELECT to_tsquery('english', '1 <3> (2 <-> a)');

SELECT to_tsquery('english', '(1 <3> 2) <-> a');

SELECT to_tsquery('english', '(1 <3> a) <-> 2');

SELECT to_tsquery('english', '(a <3> 1) <-> 2');

SELECT to_tsquery('english', 'a <-> (1 <3> 2)');

SELECT to_tsquery('english', '1 <-> (a <3> 2)');

SELECT to_tsquery('english', '1 <-> (2 <3> a)');

SELECT to_tsquery('english', '((a <-> 1) <-> 2) <-> s');

SELECT to_tsquery('english', '(2 <-> (a <-> 1)) <-> s');

SELECT to_tsquery('english', '((1 <-> a) <-> 2) <-> s');

SELECT to_tsquery('english', '(2 <-> (1 <-> a)) <-> s');

SELECT to_tsquery('english', 's <-> ((a <-> 1) <-> 2)');

SELECT to_tsquery('english', 's <-> (2 <-> (a <-> 1))');

SELECT to_tsquery('english', 's <-> ((1 <-> a) <-> 2)');

SELECT to_tsquery('english', 's <-> (2 <-> (1 <-> a))');

SELECT to_tsquery('english', '((a <-> 1) <-> s) <-> 2');

SELECT to_tsquery('english', '(s <-> (a <-> 1)) <-> 2');

SELECT to_tsquery('english', '((1 <-> a) <-> s) <-> 2');

SELECT to_tsquery('english', '(s <-> (1 <-> a)) <-> 2');

SELECT to_tsquery('english', '2 <-> ((a <-> 1) <-> s)');

SELECT to_tsquery('english', '2 <-> (s <-> (a <-> 1))');

SELECT to_tsquery('english', '2 <-> ((1 <-> a) <-> s)');

SELECT to_tsquery('english', '2 <-> (s <-> (1 <-> a))');

SELECT to_tsquery('english', 'foo <-> (a <-> (the <-> bar))');

SELECT to_tsquery('english', '((foo <-> a) <-> the) <-> bar');

SELECT to_tsquery('english', 'foo <-> a <-> the <-> bar');

SELECT phraseto_tsquery('english', 'PostgreSQL can be extended by the user in many ways');

SELECT ts_rank_cd(to_tsvector('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
'), to_tsquery('english', 'paint&water'));

SELECT ts_rank_cd(to_tsvector('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
'), to_tsquery('english', 'breath&motion&water'));

SELECT ts_rank_cd(to_tsvector('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
'), to_tsquery('english', 'ocean'));

SELECT ts_rank_cd(to_tsvector('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
'), to_tsquery('english', 'painted <-> Ship'));

SELECT ts_rank_cd(strip(to_tsvector('both stripped')),
                  to_tsquery('both & stripped'));

SELECT ts_rank_cd(to_tsvector('unstripped') || strip(to_tsvector('stripped')),
                  to_tsquery('unstripped & stripped'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'paint&water'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'breath&motion&water'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'ocean'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'day & drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'day | drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'day | !drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'painted <-> Ship & drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'painted <-> Ship | drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'painted <-> Ship | !drink'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', phraseto_tsquery('english', 'painted Ocean'));

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', phraseto_tsquery('english', 'idle as a painted Ship'));

SELECT ts_headline('english',
'Lorem ipsum urna.  Nullam nullam ullamcorper urna.',
to_tsquery('english','Lorem') && phraseto_tsquery('english','ullamcorper urna'),
'MaxWords=100, MinWords=1');

SELECT ts_headline('english',
'Lorem ipsum urna.  Nullam nullam ullamcorper urna.',
phraseto_tsquery('english','ullamcorper urna'),
'MaxWords=100, MinWords=5');

SELECT ts_headline('english', '
<html>
<!-- some comment -->
<body>
Sea view wow <u>foo bar</u> <i>qq</i>
<a href="http://www.google.com/foo.bar.html" target="_blank">YES &nbsp;</a>
ff-bg
<script>
       document.write(15);
</script>
</body>
</html>',
to_tsquery('english', 'sea&foo'), 'HighlightAll=true');

SELECT ts_headline('simple', '1 2 3 1 3'::text, '1 <-> 3', 'MaxWords=2, MinWords=1');

SELECT ts_headline('simple', '1 2 3 1 3'::text, '1 & 3', 'MaxWords=4, MinWords=1');

SELECT ts_headline('simple', '1 2 3 1 3'::text, '1 <-> 3', 'MaxWords=4, MinWords=1');

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'ocean'), 'MaxFragments=1');

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'Coleridge & stuck'), 'MaxFragments=2');

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'ocean & seahorse'), 'MaxFragments=1');

SELECT ts_headline('english', '
Day after day, day after day,
  We stuck, nor breath nor motion,
As idle as a painted Ship
  Upon a painted Ocean.
Water, water, every where
  And all the boards did shrink;
Water, water, every where,
  Nor any drop to drink.
S. T. Coleridge (1772-1834)
', to_tsquery('english', 'Coleridge & stuck'), 'MaxFragments=2,FragmentDelimiter=***');

SELECT ts_headline('english',
'Lorem ipsum urna.  Nullam nullam ullamcorper urna.',
to_tsquery('english','Lorem') && phraseto_tsquery('english','ullamcorper urna'),
'MaxFragments=100, MaxWords=100, MinWords=1');

SELECT ts_headline('english',
'', to_tsquery('english', ''));

SELECT ts_headline('english',
'foo bar', to_tsquery('english', ''));

CREATE TABLE test_tsquery (txtkeyword TEXT, txtsample TEXT);

ALTER TABLE test_tsquery ADD COLUMN keyword tsquery;

UPDATE test_tsquery SET keyword = to_tsquery('english', txtkeyword);

ALTER TABLE test_tsquery ADD COLUMN sample tsquery;

UPDATE test_tsquery SET sample = to_tsquery('english', txtsample::text);

SELECT COUNT(*) FROM test_tsquery WHERE keyword <  'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword <= 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword = 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword >= 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword >  'new <-> york';

CREATE UNIQUE INDEX bt_tsq ON test_tsquery (keyword);

SET enable_seqscan=OFF;

SELECT COUNT(*) FROM test_tsquery WHERE keyword <  'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword <= 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword = 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword >= 'new <-> york';

SELECT COUNT(*) FROM test_tsquery WHERE keyword >  'new <-> york';

RESET enable_seqscan;

SELECT ts_rewrite('foo & bar & qq & new & york',  'new & york'::tsquery, 'big & apple | nyc | new & york & city');

SELECT ts_rewrite(ts_rewrite('new & !york ', 'york', '!jersey'),
                  'jersey', 'mexico');

SELECT ts_rewrite('moscow', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite('moscow & hotel', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite('bar & qq & foo & (new <-> york)', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite( 'moscow', 'SELECT keyword, sample FROM test_tsquery');

SELECT ts_rewrite( 'moscow & hotel', 'SELECT keyword, sample FROM test_tsquery');

SELECT ts_rewrite( 'bar & qq & foo & (new <-> york)', 'SELECT keyword, sample FROM test_tsquery');

SELECT ts_rewrite('1 & (2 <-> 3)', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite('1 & (2 <2> 3)', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite('5 <-> (1 & (2 <-> 3))', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite('5 <-> (6 | 8)', 'SELECT keyword, sample FROM test_tsquery'::text );

SELECT ts_rewrite(to_tsquery('5 & (6 | 5)'), to_tsquery('5'), to_tsquery(''));

SELECT ts_rewrite(to_tsquery('!5'), to_tsquery('5'), to_tsquery(''));

SELECT keyword FROM test_tsquery WHERE keyword @> 'new';

SELECT keyword FROM test_tsquery WHERE keyword @> 'moscow';

SELECT keyword FROM test_tsquery WHERE keyword <@ 'new';

SELECT keyword FROM test_tsquery WHERE keyword <@ 'moscow';

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow & hotel') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'bar & qq & foo & (new <-> york)') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow & hotel') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'bar & qq & foo & (new <-> york)') AS query;

CREATE INDEX qq ON test_tsquery USING gist (keyword tsquery_ops);

SET enable_seqscan=OFF;

SELECT keyword FROM test_tsquery WHERE keyword @> 'new';

SELECT keyword FROM test_tsquery WHERE keyword @> 'moscow';

SELECT keyword FROM test_tsquery WHERE keyword <@ 'new';

SELECT keyword FROM test_tsquery WHERE keyword <@ 'moscow';

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow & hotel') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'bar & qq & foo & (new <-> york)') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'moscow & hotel') AS query;

SELECT ts_rewrite( query, 'SELECT keyword, sample FROM test_tsquery' ) FROM to_tsquery('english', 'bar & qq & foo & (new <-> york)') AS query;

SELECT ts_rewrite(tsquery_phrase('foo', 'foo'), 'foo', 'bar | baz');

SELECT to_tsvector('foo bar') @@
  ts_rewrite(tsquery_phrase('foo', 'foo'), 'foo', 'bar | baz');

SELECT to_tsvector('bar baz') @@
  ts_rewrite(tsquery_phrase('foo', 'foo'), 'foo', 'bar | baz');

RESET enable_seqscan;

SET default_text_search_config=simple;

SELECT to_tsvector('SKIES My booKs');

SELECT plainto_tsquery('SKIES My booKs');

SELECT to_tsquery('SKIES & My | booKs');

SET default_text_search_config=english;

SELECT to_tsvector('SKIES My booKs');

SELECT plainto_tsquery('SKIES My booKs');

SELECT to_tsquery('SKIES & My | booKs');

CREATE TRIGGER tsvectorupdate
BEFORE UPDATE OR INSERT ON test_tsvector
FOR EACH ROW EXECUTE PROCEDURE tsvector_update_trigger(a, 'pg_catalog.english', t);

SELECT count(*) FROM test_tsvector WHERE a @@ to_tsquery('345&qwerty');

INSERT INTO test_tsvector (t) VALUES ('345 qwerty');

SELECT count(*) FROM test_tsvector WHERE a @@ to_tsquery('345&qwerty');

UPDATE test_tsvector SET t = null WHERE t = '345 qwerty';

SELECT count(*) FROM test_tsvector WHERE a @@ to_tsquery('345&qwerty');

INSERT INTO test_tsvector (t) VALUES ('345 qwerty');

SELECT count(*) FROM test_tsvector WHERE a @@ to_tsquery('345&qwerty');

select * from test_tsquery, to_tsquery('new') q where txtsample @@ q;

select * from test_tsquery, to_tsquery('english', 'new') q where txtsample @@ q;

create temp table pendtest (ts tsvector);

create index pendtest_idx on pendtest using gin(ts);

insert into pendtest values (to_tsvector('Lore ipsam'));

insert into pendtest values (to_tsvector('Lore ipsum'));

select * from pendtest where 'ipsu:*'::tsquery @@ ts;

select * from pendtest where 'ipsa:*'::tsquery @@ ts;

select * from pendtest where 'ips:*'::tsquery @@ ts;

select * from pendtest where 'ipt:*'::tsquery @@ ts;

select * from pendtest where 'ipi:*'::tsquery @@ ts;

create temp table phrase_index_test(fts tsvector);

insert into phrase_index_test values ('A fat cat has just eaten a rat.');

insert into phrase_index_test values (to_tsvector('english', 'A fat cat has just eaten a rat.'));

create index phrase_index_test_idx on phrase_index_test using gin(fts);

set enable_seqscan = off;

select * from phrase_index_test where fts @@ phraseto_tsquery('english', 'fat cat');

set enable_seqscan = on;

select websearch_to_tsquery('simple', 'I have a fat:*ABCD cat');

select websearch_to_tsquery('simple', 'orange:**AABBCCDD');

select websearch_to_tsquery('simple', 'fat:A!cat:B|rat:C<');

select websearch_to_tsquery('simple', 'fat:A : cat:B');

select websearch_to_tsquery('simple', 'fat*rat');

select websearch_to_tsquery('simple', 'fat-rat');

select websearch_to_tsquery('simple', 'fat_rat');

select websearch_to_tsquery('simple', 'abc : def');

select websearch_to_tsquery('simple', 'abc:def');

select websearch_to_tsquery('simple', 'a:::b');

select websearch_to_tsquery('simple', 'abc:d');

select websearch_to_tsquery('simple', ':');

select websearch_to_tsquery('simple', 'abc & def');

select websearch_to_tsquery('simple', 'abc | def');

select websearch_to_tsquery('simple', 'abc <-> def');

select websearch_to_tsquery('simple', 'abc (pg or class)');

select websearch_to_tsquery('simple', '(foo bar) or (ding dong)');

select websearch_to_tsquery('english', 'My brand new smartphone');

select websearch_to_tsquery('english', 'My brand "new smartphone"');

select websearch_to_tsquery('english', 'My brand "new -smartphone"');

select websearch_to_tsquery('simple', 'cat or rat');

select websearch_to_tsquery('simple', 'cat OR rat');

select websearch_to_tsquery('simple', 'cat "OR" rat');

select websearch_to_tsquery('simple', 'cat OR');

select websearch_to_tsquery('simple', 'OR rat');

select websearch_to_tsquery('simple', '"fat cat OR rat"');

select websearch_to_tsquery('simple', 'fat (cat OR rat');

select websearch_to_tsquery('simple', 'or OR or');

select websearch_to_tsquery('simple', '"fat cat"or"fat rat"');

select websearch_to_tsquery('simple', 'fat or(rat');

select websearch_to_tsquery('simple', 'fat or)rat');

select websearch_to_tsquery('simple', 'fat or&rat');

select websearch_to_tsquery('simple', 'fat or|rat');

select websearch_to_tsquery('simple', 'fat or!rat');

select websearch_to_tsquery('simple', 'fat or<rat');

select websearch_to_tsquery('simple', 'fat or>rat');

select websearch_to_tsquery('simple', 'fat or ');

select websearch_to_tsquery('simple', 'abc orange');

select websearch_to_tsquery('simple', 'abc OR1234');

select websearch_to_tsquery('simple', 'abc or-abc');

select websearch_to_tsquery('simple', 'abc OR_abc');

select websearch_to_tsquery('english', '"pg_class pg');

select websearch_to_tsquery('english', 'pg_class pg"');

select websearch_to_tsquery('english', '"pg_class pg"');

select websearch_to_tsquery('english', '"pg_class : pg"');

select websearch_to_tsquery('english', 'abc "pg_class pg"');

select websearch_to_tsquery('english', '"pg_class pg" def');

select websearch_to_tsquery('english', 'abc "pg pg_class pg" def');

select websearch_to_tsquery('english', ' or "pg pg_class pg" or ');

select websearch_to_tsquery('english', '""pg pg_class pg""');

select websearch_to_tsquery('english', 'abc """"" def');

select websearch_to_tsquery('english', 'cat -"fat rat"');

select websearch_to_tsquery('english', 'cat -"fat rat" cheese');

select websearch_to_tsquery('english', 'abc "def -"');

select websearch_to_tsquery('english', 'abc "def :"');

select websearch_to_tsquery('english', '"A fat cat" has just eaten a -rat.');

select websearch_to_tsquery('english', '"A fat cat" has just eaten OR !rat.');

select websearch_to_tsquery('english', '"A fat cat" has just (+eaten OR -rat)');

select websearch_to_tsquery('english', 'this is ----fine');

select websearch_to_tsquery('english', '(()) )))) this ||| is && -fine, "dear friend" OR good');

select websearch_to_tsquery('english', 'an old <-> cat " is fine &&& too');

select websearch_to_tsquery('english', '"A the" OR just on');

select websearch_to_tsquery('english', '"a fat cat" ate a rat');

select to_tsvector('english', 'A fat cat ate a rat') @@
	websearch_to_tsquery('english', '"a fat cat" ate a rat');

select to_tsvector('english', 'A fat grey cat ate a rat') @@
	websearch_to_tsquery('english', '"a fat cat" ate a rat');

select websearch_to_tsquery('''');

select websearch_to_tsquery('''abc''''def''');

select websearch_to_tsquery('\abc');

select websearch_to_tsquery('\');
