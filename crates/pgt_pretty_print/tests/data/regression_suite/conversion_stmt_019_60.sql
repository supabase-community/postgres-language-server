select description, (test_conv(inbytes, 'utf8', 'utf8')).* from utf8_verification_inputs;
