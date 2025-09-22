insert into mic_inputs  values
  ('\x666f6f',		'valid, pure ASCII'),
  ('\x8bc68bcf8bcf',	'valid (in KOI8R)'),
  ('\x8bc68bcf8b',	'invalid,incomplete char'),
  ('\x92bedd',		'valid (in SHIFT_JIS)'),
  ('\x92be',		'invalid, incomplete char)'),
  ('\x666f6f95a3c1',	'valid (in Big5)'),
  ('\x666f6f95a3',	'invalid, incomplete char'),
  ('\x9200bedd',	'invalid, NUL byte'),
  ('\x92bedd00',	'invalid, NUL byte'),
  ('\x8b00c68bcf8bcf',	'invalid, NUL byte');
