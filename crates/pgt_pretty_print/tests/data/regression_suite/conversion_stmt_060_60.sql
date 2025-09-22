select description, inbytes, (test_conv(inbytes, 'mule_internal', 'koi8r')).* from mic_inputs;
