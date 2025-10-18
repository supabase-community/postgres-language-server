CREATE TEMP TABLE x (
	a serial,
	b int,
	c text not null default 'stuff',
	d text,
	e text
) ;

CREATE FUNCTION fn_x_before () RETURNS TRIGGER AS '
  BEGIN
		NEW.e := ''before trigger fired''::text;
		return NEW;
	END;
' LANGUAGE plpgsql;

CREATE FUNCTION fn_x_after () RETURNS TRIGGER AS '
  BEGIN
		UPDATE x set e=''after trigger fired'' where c=''stuff'';
		return NULL;
	END;
' LANGUAGE plpgsql;

CREATE TRIGGER trg_x_after AFTER INSERT ON x
FOR EACH ROW EXECUTE PROCEDURE fn_x_after();

CREATE TRIGGER trg_x_before BEFORE INSERT ON x
FOR EACH ROW EXECUTE PROCEDURE fn_x_before();

SELECT * FROM x;

COPY x TO stdout;

COPY x (c, e) TO stdout;

COPY x (b, e) TO stdout
