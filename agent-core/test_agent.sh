#!/bin/bash

echo "=== Testing Orban Agent CLI ==="
echo ""

echo "1️⃣  Testing version command:"
./target/release/orban-agent version
echo ""

echo "2️⃣  Testing status (before start):"
./target/release/orban-agent status
echo ""

echo "3️⃣  Testing help:"
./target/release/orban-agent --help
echo ""

