FROM ubuntu:latest

# RUN apk add --no-cache --update curl git gcc build-base alpine-sdk
RUN apt-get update
RUN apt-get -y install curl unzip
RUN curl -LJO https://github.com/Unreal-Engine-FR/Bidibip/releases/latest/download/bidibip_linux.zip
RUN unzip -q ./bidibip_linux.zip
RUN mkdir /opt/bidibip
RUN mv ./bidibip/bidibip-core /opt/bidibip/bidibip
RUN rm ./bidibip_linux.zip
RUN rmdir ./bidibip
RUN chmod a+x /opt/bidibip/bidibip

CMD ["/opt/bidibip/bidibip"]