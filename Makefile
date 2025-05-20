build: prebuild
	npm run tauri build

dev: prebuild
	npm run tauri dev

%/.git:
	git submodule update --init --recursive

# TODO: add icon generation
# src-tauri/icons/icon.png:

gptme-webui/dist: gptme-webui/.git
	cd gptme-webui && npm run build

prebuild: gptme-webui/dist src-tauri/icons/icon.png

precommit: format check

format:
	cd src-tauri && cargo fmt

check:
	cd src-tauri && cargo check && cargo clippy
