SELECT getdatabaseencoding() <> 'UTF8'
       AS skip_test ;

CREATE TABLE copy_encoding_tab (t text);

COPY (SELECT E'\u3042') TO 'utf8_csv'

COPY copy_encoding_tab FROM 'utf8_csv'

SET client_encoding TO UTF8;

COPY (SELECT E'\u3042') TO 'utf8_csv'

SET client_encoding TO LATIN1;

COPY copy_encoding_tab FROM 'utf8_csv'

RESET client_encoding;

COPY (SELECT E'\u3042') TO 'utf8_csv'

COPY copy_encoding_tab FROM 'utf8_csv'

SET client_encoding TO UTF8;

COPY (SELECT E'\u3042') TO 'utf8_csv'

SET client_encoding TO EUC_JP;

COPY copy_encoding_tab FROM 'utf8_csv'

RESET client_encoding;

DROP TABLE copy_encoding_tab;
