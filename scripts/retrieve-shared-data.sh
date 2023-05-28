#!/bin/sh

echo 'Total disk usage :' `sudo du -hs /var/lib/docker/volumes/bidibip-saved/_data/ | awk '{ print $1 }'`

echo "Are you sure you wants to retrieve a local copy of bidibip's data ? (it will duplicate it)"
read -p "Continue (y/N)?" choice
case "$choice" in
  y|Y )
    sudo rm -r /home/ubuntu/docker/Bidibip/saved/
    mkdir -p /home/ubuntu/docker/Bidibip/saved/
    sudo cp -r /var/lib/docker/volumes/bidibip-saved/_data/ /home/ubuntu/docker/Bidibip/saved/
    sudo mv /home/ubuntu/docker/Bidibip/saved/_data/* /home/ubuntu/docker/Bidibip/saved
    sudo rmdir /home/ubuntu/docker/Bidibip/saved/_data
    echo "Successfully moved saved dir. Disk usage :"
    du -h /home/ubuntu/docker/Bidibip/saved/;;
  * ) return 0;;
esac