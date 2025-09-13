# Athena CLI Makefile

.PHONY: help build test install uninstall clean dev

help: ## Show help
	@echo "Athena CLI - Available Commands:"
	@echo "=================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in release mode
	@echo "📦 Building project..."
	cargo build --release
	@echo "✅ Build complete: target/release/athena"

test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test
	@echo "✅ Tests completed"

install: build ## Install Athena locally
	@echo "🚀 Installing Athena..."
	cargo install --path . --force
	@echo "✅ Installation complete!"
	@echo "Try with: athena --help"

install-system: build ## System-wide installation (requires sudo)
	@echo "🚀 Installing Athena system-wide..."
	sudo cp target/release/athena /usr/local/bin/
	sudo chmod +x /usr/local/bin/athena
	@echo "✅ Athena installed in /usr/local/bin/"
	@echo "Try with: athena --help"

uninstall: ## Uninstall Athena
	@echo "🗑️  Uninstalling Athena..."
	-cargo uninstall athena
	-sudo rm -f /usr/local/bin/athena
	@echo "✅ Athena uninstalled"

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	cargo clean
	rm -f docker-compose.yml my-compose.yml production-compose.yml
	rm -f test-*.ath demo-*.ath
	@echo "✅ Clean complete"

dev: build ## Development mode with tests
	@echo "🔧 Development mode..."
	cargo test
	./target/release/athena info
	@echo "✅ Ready for development"

demo: install ## Installation + full demo
	@echo "🎯 Athena demo..."
	@echo "DEPLOYMENT-ID DEMO\n\nSERVICES SECTION\n\nSERVICE web\nIMAGE-ID nginx:alpine\nPORT-MAPPING 80 TO 80\nEND SERVICE" > demo.ath
	athena --verbose build demo.ath
	@echo "✅ Demo complete! File: docker-compose.yml"

check-install: ## Verify installation
	@echo "🔍 Checking installation..."
	@if command -v athena >/dev/null 2>&1; then \
		echo "✅ Athena is installed:"; \
		which athena; \
		athena --help | head -3; \
	else \
		echo "❌ Athena is not installed or not in PATH"; \
		echo "Run: make install"; \
	fi
