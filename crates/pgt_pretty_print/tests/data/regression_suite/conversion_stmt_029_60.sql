select description, inbytes, (test_conv(inbytes, 'utf8', 'latin2')).* from utf8_inputs;
