#!/bin/bash

# Start Xvfb
Xvfb :99 -ac &

# Run the main command (in this case, your Rust application)
exec "$@"