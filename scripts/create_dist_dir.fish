cargo build --release
rm -rf dist
mkdir -p dist
mkdir -p dist/sql
cp target/release/sensor_data_server dist/sensor_data_server
cp cfg.toml dist/cfg.toml
cp -r sql/ dist/
