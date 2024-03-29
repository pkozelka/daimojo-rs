all: build

objdump:
	objdump -p lib/linux_x64/libdaimojo.so

needed:
	objdump -p lib/linux_x64/libdaimojo.so | grep NEEDED

target/libdemo.so:
	cc src/demo.c -shared -o target/libdemo.so

to-rust:
	sed -n --file=to-rust.sed lib/linux_x64/c_api.h > target/capi.txt

test-wine:
	cargo run -- --mojo data/wine/pipeline.mojo predict data/wine/wine_test.csv

info-wine:
	cargo run -- --mojo data/wine/pipeline.mojo show

test-str:
	cargo run -- --mojo data/fillna_str/pipeline.mojo predict data/fillna_str/example.csv

test-simple:
	cargo run -- -vvv --mojo tests/data/transform_agg_sum_py.mojo predict tests/data/transform_agg_sum_py.input.csv
