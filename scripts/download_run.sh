#!/bin/sh

wget -O /opt/bidibip/bidibip.zip https://github.com/Unreal-Engine-FR/Bidibip/releases/download/v4.0.8/bidibip_linux.zip
unzip /opt/bidibip/bidibip.zip -d /opt/bidibip
rm /opt/bidibip/bidibip.zip
mv /opt/bidibip/bidibip/bidibip-core /opt/bidibip/bidibip-core
rmdir /opt/bidibip/bidibip/
chmod u+x /opt/bidibip/bidibip-core
/opt/bidibip/bidibip-core /saved/config.json