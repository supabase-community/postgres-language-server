select description, inbytes, (test_conv(inbytes, 'utf8', 'koi8r')).* from utf8_inputs;
