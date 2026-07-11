#!/bin/bash
# Cleans the garbage files, not the important compiled libraries
find target/debug/deps -name ".mold-*" -type f -delete
echo "cleaned mold artifacts"