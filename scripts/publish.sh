#!/bin/bash

echo "Building a release version of the project..."
cargo build --release

echo "Copying the binary to the target directory..."
echo "Current directory: $(pwd)"
sudo cp target/release/drem /usr/local/bin/drem

echo "Done!"
