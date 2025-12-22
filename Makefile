.PHONY: build release clean run

# Use Windows cargo.exe from WSL
CARGO := cargo.exe

# Default target
build:
	$(CARGO) build

release:
	$(CARGO) build --release

clean:
	$(CARGO) clean

run:
	$(CARGO) run
