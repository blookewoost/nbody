#!/bin/bash

# Three-Body Simulator and Viewer Workflow Script
# Usage: ./simulate_and_view.sh <config.ini> [output.csv]
# Example: ./simulate_and_view.sh data/earth_moon.ini

set -e  # Exit on error

# Check if config file is provided
if [ $# -lt 1 ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    echo "Usage: $0 <config.ini> [output.csv]"
    echo ""
    echo "Runs a three-body simulation with the given config file and launches the viewer."
    echo ""
    echo "Arguments:"
    echo "  <config.ini>   Path to the configuration file (required)"
    echo "  [output.csv]   Output file for results (default: data/results.csv)"
    echo ""
    echo "Example:"
    echo "  $0 data/earth_moon.ini"
    echo "  $0 data/binary_stars.ini data/custom_results.csv"
    exit 0
fi

CONFIG_FILE="$1"
OUTPUT_FILE="${2:-data/results.csv}"

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Config file '$CONFIG_FILE' not found"
    exit 1
fi

echo "=========================================="
echo "Three-Body Simulator & Viewer"
echo "=========================================="
echo ""
echo "Config file: $CONFIG_FILE"
echo "Output file: $OUTPUT_FILE"
echo ""

# Run the simulator
echo "Running simulation..."
cargo run --release --bin threebody-sim -- "$CONFIG_FILE" "$OUTPUT_FILE"

if [ ! -f "$OUTPUT_FILE" ]; then
    echo "Error: Simulation failed, output file not created"
    exit 1
fi

echo ""
echo "Simulation complete!"
echo "Launching viewer..."
echo ""

# Run the viewer
cargo run --release --bin viewer --features="viewer" -- "$OUTPUT_FILE"
