SELECT string_to_array('ABC,DEF,GHI'::char(11) COLLATE case_insensitive, ',', 'abc');
