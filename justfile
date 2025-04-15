build-release:
  cargo build --release
  cp target/release/etl_gateway release/

run:
  cargo build
  export ETL_GATEWAY_DEBUG=true && ./run-etl.sh
