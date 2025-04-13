#!/bin/sh

wget -O /opt/bidibip/bidibip.zip https://github.com/Unreal-Engine-FR/Bidibip/releases/latest/download/bidibip_linux.zip
unzip /opt/bidibip/bidibip.zip -d /opt/bidibip
rm /opt/bidibip/bidibip.zip
mv /opt/bidibip/bidibip/bidibip-core /opt/bidibip/bidibip_core
rmdir /opt/bidibip/bidibip/
chmod u+x /opt/bidibip/bidibip_core
/opt/bidibip/bidibip_core /saved/config.json