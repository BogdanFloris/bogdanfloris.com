APP_NAME=bogdanfloris-com
VERSION?=v0.1.0
BUILD=$(shell git rev-parse HEAD)

CARGO=~/.cargo/bin/cargo
TAILWIND=/usr/local/bin/tailwindcss

.DEFAULT_GOAL := help

.PHONY: help
# Source: https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
help: ## Displays all the available commands
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: fmt
fmt: ## Format the project
	@$(CARGO) fmt

.PHONY: test
test: ## Runs tests
	@$(CARGO) test

.PHONY: clean
clean: ## Deletes all compiled / executable files
	@$(CARGO) clean

.PHONY: run
run: ## Runs the backend server
	@$(CARGO) run

.PHONY: dev
dev: ## Runs the backend server with hot-reload (Must have cargo watch installed)
	@$(CARGO) watch -x run

.PHONY: tailwind
tailwind: ## Runs the tailwind compile command with --watch flag
	@$(TAILWIND) -i src/style.css -o dist/output.css --watch

.PHONY: build
build: ## Compiles the server
	@${TAILWIND} -i src/style.css -o dist/output.css
	@$(CARGO) build

.PHONY: build-release
build-release: ## Compiles the server with release flag
	@${TAILWIND} -i src/style.css -o dist/output.css
	@$(CARGO) build --release
	
