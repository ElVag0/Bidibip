#!/bin/sh

wget -O /opt/bidibip/bidibip.zip https://github.com/Unreal-Engine-FR/Bidibip/releases/latest/download/bidibip_linux.zip
unzip /opt/bidibip/bidibip.zip -d /opt/bidibip
rm /opt/bidibip/bidibip.zip
mv /opt/bidibip/bidibip/bidibip /opt/bidibip/bidibip_exe
rmdir /opt/bidibip/bidibip/
chmod u+x /opt/bidibip/bidibip_exe
/opt/bidibip/bidibip_exe /saved/config.json