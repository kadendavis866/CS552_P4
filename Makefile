export PATH := $(HOME)/.cargo/bin:$(PATH)
all:
	cargo build
	cp target/debug/P4 ./
check:
	cargo test -- --test-threads=1

.PHONY: clean
clean:
	cargo clean
	rm -f ./P4

.PHONY: install-deps
install-deps:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	export PATH="$$HOME/.cargo/bin:$$PATH"