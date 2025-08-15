# Actix Todo API - Developer Makefile

APP_NAME := actix-todo-api
BIN := target/release/$(APP_NAME)
PORT ?= 8080
DATABASE_URL ?= sqlite:data/todos.db

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make run                - Run the service (debug)"
	@echo "  make build              - Build release binary"
	@echo "  make test               - Run all unit + integration tests"
	@echo "  make fmt                - Format code"
	@echo "  make clippy             - Lint with clippy"
	@echo "  make coverage           - Run coverage with cargo-llvm-cov (HTML report)"
	@echo "  make docker-build       - Build Docker image"
	@echo "  make docker-run         - Run Docker container"
	@echo "  make clean              - Clean target directory"

.PHONY: run
run:
	RUST_LOG=info,actix_web=info PORT=$(PORT) DATABASE_URL=$(DATABASE_URL) cargo run

.PHONY: build
build:
	cargo build --release

.PHONY: test
test:
	# Run all tests with output
	RUST_LOG=warn cargo test -- --nocapture

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: coverage
coverage:
	# Requires cargo-llvm-cov: install with `cargo install cargo-llvm-cov`
	cargo llvm-cov clean --workspace
	RUST_LOG=warn cargo llvm-cov --workspace --lcov --html --ignore-filename-regex '(.*/src/main.rs)'

.PHONY: docker-build
docker-build:
	docker build -t $(APP_NAME):latest .

.PHONY: docker-run
docker-run:
	docker run --rm -p $(PORT):8080 -e PORT=8080 -e DATABASE_URL=$(DATABASE_URL) $(APP_NAME):latest

.PHONY: clean
clean:
	cargo clean