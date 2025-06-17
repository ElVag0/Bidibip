#!/bin/sh

VERSION=${1:-latest}

# Remove old binary
rm -rf /opt/bidibip/bidibip

# Download and extract desired bidibip version
wget -O /opt/bidibip/bidibip.zip https://github.com/Unreal-Engine-FR/Bidibip/releases/latest/download/bidibip_linux.zip
unzip /opt/bidibip/bidibip.zip -d /opt

# Install
chmod u+x /opt/bidibip/bidibip

# Cleanup
rm /opt/bidibip/bidibip.zip

# Run
/opt/bidibip/bidibip