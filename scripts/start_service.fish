function kill_process
	pkill -f -u $USER $argv
	if test $status -eq 0; then
		echo "Successfully shutdown old running service $argv.'"
	else if test $status -eq 1; then
		echo 'Could not find or shutdown old running service ${argv}.'
	else
		echo 'An error occured while trying to shutdown service ${argv}.'
	end
end

kill_process 'sensor_data_server'

echo 'Enabling posgreql service.'
sudo systemctl enable postgresql.service
set -gx DATABASE_URL "postgres://user:password@localhost/schili_sensor_db" 

pkill -f -u $USER 'mosquitto'
if test $status -eq 0; then
	echo 'Mosquitto mqtt broker already running.'
else 
	echo 'Starting mosquitto mqtt broker.'
        mosquitto -c mosquitto.conf &
end
./dist/sensor_data_server &
