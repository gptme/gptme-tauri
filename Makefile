build: prebuild
	@# Set NO_STRIP=true on Linux to avoid "failed to run linuxdeploy" error
	@if [ "$$(uname)" = "Linux" ]; then \
		echo "Building on Linux, setting NO_STRIP=true..."; \
		NO_STRIP=true npm run tauri build; \
	else \
		echo "Building on non-Linux platform..."; \
		npm run tauri build; \
	fi

dev: prebuild
	npm run tauri dev

%/.git:
	git submodule update --init --recursive

src-tauri/icons/icon.png:
	npm run tauri icon "./public/logo.png"

gptme-webui/dist: gptme-webui/.git
	# TODO: probably a better way to do this
	npm i && cd gptme-webui && npm i && npm run build

gptme-server-build:
	@if [ ! -d "bins" ]; then \
		echo "Building gptme-server..."; \
		mkdir -p bins; \
		cd gptme && make build-server-exe && mv dist/gptme-server ../bins/gptme-server-$$(rustc -Vv | grep host | cut -f2 -d' '); \
	else \
		echo "bins folder already exists, skipping gptme-server build"; \
	fi

prebuild: gptme-webui/dist src-tauri/icons/icon.png
	@$(MAKE) gptme-server-build

precommit: format check

format:
	cd src-tauri && cargo fmt

check:
	cd src-tauri && cargo check && cargo clippy
