FROM ubuntu:latest

RUN apt-get update
RUN apt-get -y install wget unzip
RUN mkdir /opt/bidibip
COPY ./scripts/download_run.sh /opt/bidibip
CMD ["/opt/bidibip/download_run.sh"]