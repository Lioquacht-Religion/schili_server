
CREATE TYPE sensor_type AS ENUM ('temperature', 'humidity', 'airpressure', 'co2');

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

CREATE TABLE co2 (
	co2_id bigint PRIMARY KEY
	GENERATED ALWAYS AS IDENTITY,
	sensor_id integer 
	REFERENCES sensors (sensor_id) NOT NULL,
	--TODO: add unit
	co2_unit numeric(6, 3) NOT NULL,
	measure_time timestamp NOT NULL
);

