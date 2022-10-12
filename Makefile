all: build

objdump:
	objdump -p lib/linux_x64/libdaimojo.so

needed:
	objdump -p lib/linux_x64/libdaimojo.so | grep NEEDED

target/libdemo.so:
	cc src/demo.c -shared -o target/libdemo.so

to-rust:
	sed -n --file=to-rust.sed lib/linux_x64/c_api.h > target/capi.txt

