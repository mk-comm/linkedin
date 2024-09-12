#!/bin/bash

echo "Starting Xvfb..."
Xvfb :99 -screen 0 1920x1080x24 &
echo "Xvfb started."

echo "Setting DISPLAY variable..."
export DISPLAY=:99

echo "Running web crawler application..."
/app/web_crawler
echo "Web crawler finished."
