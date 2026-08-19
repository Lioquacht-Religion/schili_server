function kill_process
	pkill -f -u $USER $argv
	if test $status -eq 0
		echo "Successfully shutdown old running service $argv."
	else if test $status -eq 1
		echo "Could not find or shutdown old running service $argv."
	else
		echo "An error occured while trying to shutdown service $argv."
	end
end

kill_process 'sensor_data_server'
kill_process 'mosquitto'
