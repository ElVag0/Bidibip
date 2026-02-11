#!Dockerfile

FROM alpine:latest

ARG VERSION=latest

RUN apk update
RUN apk add wget unzip dos2unix
RUN mkdir /opt/bidibip

COPY scripts/config.json /opt/bidibip/
COPY scripts/download_run.sh /opt/bidibip/

RUN chmod u+x /opt/bidibip/download_run.sh
RUN dos2unix /opt/bidibip/download_run.sh

CMD /opt/bidibip/download_run.sh ${VERSION}