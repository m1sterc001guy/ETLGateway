build-release:
  cargo build --release
  cp target-nix/release/etl_gateway release/

run:
  cargo build
  export ETL_GATEWAY_DEBUG=true && ./run-etl.sh