.PHONY: help up down postgres api-dev db-migrate db-seed api-build api-test api-lint api-coverage

help:
	@echo "Ficus monorepo commands"
	@echo "  make up              Start postgres (lightweight dev)"
	@echo "  make up-full         Start all services including API"
	@echo "  make up-obs          Start observability stack"
	@echo "  make down            Stop all services"
	@echo "  make db-migrate      Run database migrations"
	@echo "  make db-seed         Seed development users"
	@echo "  make api-dev         Run API locally"
	@echo "  make api-build       Build API release"
	@echo "  make api-test        Run API tests"
	@echo "  make api-lint        Run Rust fmt + clippy"
	@echo "  make api-coverage    Run API coverage"

up:
	docker compose up -d postgres

up-full:
	docker compose --profile full up -d --build

up-obs:
	docker compose --profile observability up -d

down:
	docker compose --profile full --profile observability down

postgres:
	docker compose up -d postgres

db-migrate:
	cd apps/api && cargo run -p ficus-infrastructure --bin migrate

db-seed:
	cd apps/api && cargo run -p ficus-infrastructure --bin seed

api-dev:
	cd apps/api && cargo run -p ficus-infrastructure --bin ficus-api

api-build:
	cd apps/api && cargo build --release -p ficus-infrastructure

api-test:
	cd apps/api && cargo nextest run --workspace

api-lint:
	cd apps/api && cargo fmt --check
	cd apps/api && cargo clippy --workspace --all-targets --all-features -- -D warnings

api-coverage:
	cd apps/api && cargo llvm-cov --workspace --all-features --fail-under-lines 90 --fail-under-functions 90
