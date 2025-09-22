SELECT segment_number, file_offset
FROM pg_walfile_name_offset('0/0'::pg_lsn + 'segment_size'),
     pg_split_walfile_name(file_name);
