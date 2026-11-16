
CREATE TYPE sensor_type AS ENUM ('temperature', 'humidity', 'airpressure', 'co2', 'chiptemperature');

ALTER TYPE sensor_type ADD VALUE 'chiptemperature' AFTER 'co2';

ALTER TYPE sensor_type ADD VALUE 'batteryvoltage' AFTER 'chiptemperature';

ALTER TYPE sensor_type ADD VALUE 'lightintensity' AFTER 'airpressure';

CREATE TABLE sensors(
	sensor_id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	sensor_reference text UNIQUE NOT NULL,
	sensor_name text NOT NULL
);

CREATE TABLE sensor_types_link (
	sensor_types_link_id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) ON DELETE CASCADE NOT NULL,
	sensor_type sensor_type NOT NULL
);

ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_high_num numeric;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_high_timeout interval;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_low_num numeric;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_low_timeout interval;

ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_increase_num numeric;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_increase_search_interval interval;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_decrease_num numeric;
ALTER TABLE sensor_types_link ADD COLUMN IF NOT EXISTS warn_decrease_search_interval interval;

ALTER TABLE sensor_types_link DROP COLUMN IF EXISTS warn_high_increase_num;
ALTER TABLE sensor_types_link DROP COLUMN IF EXISTS warn_high_increase_search_interval;
ALTER TABLE sensor_types_link DROP COLUMN IF EXISTS warn_low_increase_num;
ALTER TABLE sensor_types_link DROP COLUMN IF EXISTS warn_low_increase_search_interval;

CREATE TABLE chip_temperatures (
	chip_temperature_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	temp_celsius numeric(6, 3) NOT NULL,
	measure_time timestamp NOT NULL
);

CREATE TABLE temperatures (
	temperature_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	temp_celsius numeric(6, 3) NOT NULL,
	measure_time timestamp NOT NULL
);

CREATE TABLE humidities (
	humidity_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	humidity_percent numeric(6, 3) NOT NULL,
	measure_time timestamp NOT NULL
);

CREATE TABLE air_pressures (
	air_pressure_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	air_pressure_pa numeric(6, 3) NOT NULL,
	measure_time timestamp NOT NULL
);

ALTER TABLE air_pressures ALTER COLUMN air_pressure_pa TYPE numeric(32,9);

CREATE TABLE light_intensities(
	light_intensity_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	light_intensity numeric(6, 9) NOT NULL,
	measure_time timestamp NOT NULL
);

ALTER TABLE light_intensities ALTER COLUMN light_intensity TYPE numeric(32,9);

CREATE TABLE battery_voltages(
	battery_voltage_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	battery_volt numeric(9, 6) NOT NULL,
	measure_time timestamp NOT NULL
);

CREATE TABLE co2 (
	co2_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	co2_ppm numeric(9, 3) NOT NULL,
	res0 numeric(15, 5) NOT NULL,
	adc_val_12bit integer NOT NULL,
	measure_time timestamp NOT NULL
);

CREATE TABLE sensor_errors(
	sensor_error_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	error_code integer NOT NULL,
	error_text text NOT NULL,
	error_time timestamp NOT NULL
);

CREATE TABLE sensor_measurements(
	sensor_measurement_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	measure_type sensor_type NOT NULL,
	measurement numeric(32, 9) NOT NULL,
	measure_time timestamp NOT NULL
);

INSERT INTO sensor_measurements (sensor_id, measure_type, measurement, measure_time)
SELECT sensor_id, 'temperature'::sensor_type, temp_celsius, measure_time
FROM temperatures
UNION
SELECT sensor_id, 'humidity'::sensor_type, humidity_percent, measure_time
FROM humidities
UNION
SELECT sensor_id, 'chiptemperature'::sensor_type, temp_celsius, measure_time
FROM chip_temperatures
UNION
SELECT sensor_id, 'airpressure'::sensor_type, air_pressure_pa, measure_time
FROM air_pressures
UNION
SELECT sensor_id, 'batteryvoltage'::sensor_type, battery_volt, measure_time
FROM battery_voltages
UNION
SELECT sensor_id, 'lightintensity'::sensor_type, light_intensity, measure_time
FROM light_intensities
;
