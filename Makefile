PHONY: .all
.all: build

.PHONY: format
format:
	cargo fmt

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: build
build:
	cargo build

.PHONY: check
check:
	cargo check

.PHONY: install-user
install-user: build
	mkdir -p ~/.purple/plugins
	cp target/debug/libpurple_icq.so ~/.purple/plugins/libpurple_icq.so

.PHONY: uninstall-user
uninstall-user:
	rm -f ~/.purple/plugins/libpurple_icq.so

.PHONY:
uninstall:
	rm -f /usr/lib/purple-2/libpurple_icq.so

.PHONY:
install: build
	mkdir -p /usr/lib/purple-2/
	cp target/debug/libpurple_icq.so /usr/lib/purple-2/libpurple_icq.so
