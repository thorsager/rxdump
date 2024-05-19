.PHONY: build clean
build:
	cargo build

release:
	cargo build --release

clean:
	rm -rf target
