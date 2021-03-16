all: ./target/x86_64-unknown-linux-musl/release/ef

./target/x86_64-unknown-linux-musl/release/ef: $(shell find . -type f -iname '*.rs' -o -name 'Cargo.toml' | sed 's/ /\\ /g') $(shell find ./resources -type f | sed 's/ /\\ /g')
	cargo build --release --target x86_64-unknown-linux-musl
	strip ./target/x86_64-unknown-linux-musl/release/ef
	
install:
	$(MAKE)
	sudo cp ./target/x86_64-unknown-linux-musl/release/ef /usr/local/bin/ef
	sudo chown root: /usr/local/bin/ef
	sudo chmod 0755 /usr/local/bin/ef

uninstall:
	sudo rm /usr/local/bin/ef

test:
	cargo test --verbose

clean:
	cargo clean
