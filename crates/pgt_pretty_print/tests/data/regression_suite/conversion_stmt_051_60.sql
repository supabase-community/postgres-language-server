select description, inbytes, (test_conv(inbytes, 'iso8859_5', 'mule_internal')).* from iso8859_5_inputs;
