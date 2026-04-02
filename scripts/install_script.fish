git pull
cargo build --release
fish scripts/create_dist_dir.fish
psql -d schili_sensor_db -a -f "sql/sensors.sql"


