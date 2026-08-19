cd ../schili_api
git pull origin main
cd ../schili_frontend
git pull origin main
cd ../schili_server
git pull origin main
cargo build --release
fish scripts/create_dist_dir.fish
psql -d schili_sensor_db -a -f "sql/sensors.sql"


