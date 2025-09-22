select description, inbytes, (test_conv(inbytes, 'utf8', 'latin1')).* from utf8_inputs;
