CREATE POLICY measurement_p ON measurement USING (peaktemp IS NOT NULL);
