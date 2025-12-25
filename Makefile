.PHONY: build serve dev migrate help run clean

# Default target
help:
	@echo "Available commands:"
	@echo "  make build    - Compile the project"
	@echo "  make serve    - Run the server (cargo run)"
	@echo "  make run      - Alias for serve"
	@echo "  make dev      - Run with hot-reload (cargo-watch)"
	@echo "  make migrate  - Run database migrations"
	@echo "  make clean    - Clean cargo build artifacts"

build:
	cargo build

serve:
	cargo run

run: serve

dev:
	bash scripts/dev.sh

migrate:
	bash scripts/migrate.sh

clean:
	cargo clean
