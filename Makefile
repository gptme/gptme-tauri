build: prebuild
	npm run tauri build

dev: prebuild
	npm run tauri dev

%/.git:
	git submodule update --init --recursive

src-tauri/icons/icon.png:
	npm run tauri icon "./public/logo.png"

gptme-webui/dist: gptme-webui/.git
	# TODO: probably a better way to do this
	npm i && cd gptme-webui && npm i && npm run build

prebuild: gptme-webui/dist src-tauri/icons/icon.png

precommit: format check

format:
	cd src-tauri && cargo fmt

check:
	cd src-tauri && cargo check && cargo clippy
