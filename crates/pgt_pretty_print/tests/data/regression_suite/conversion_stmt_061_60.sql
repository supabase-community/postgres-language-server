select description, inbytes, (test_conv(inbytes, 'mule_internal', 'iso8859-5')).* from mic_inputs;
