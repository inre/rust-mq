BIN 			?= /usr/local/bin
TARGETDIR ?= target/release
CLIENT 		?= mqttc

build:
	@cargo build --release

test:
	@RUST_TEST_THREADS=1 cargo test

docs: build
	@cargo doc --no-deps

clean:
	@cargo clean

install:
	@install $(TARGETDIR)/$(CLIENT) $(BIN)

uninstall:
	rm -rf $(BIN)/$(CLIENT)
