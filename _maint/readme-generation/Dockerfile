FROM ubuntu:16.04

ARG FF_TAG_NAME
ENV FF_USER ff-demo

RUN apt update
RUN apt install -y \
    git \
    wget

ADD _utils.sh  .

ADD generate-readme-for-init.sh .
RUN adduser --disabled-password --gecos "" $FF_USER
USER $FF_USER
RUN cd /home/$FF_USER && bash /generate-readme-for-init.sh

USER root
RUN userdel -r $FF_USER

ADD generate-readme-for-apply.sh .
RUN adduser --disabled-password --gecos "" $FF_USER
USER $FF_USER
RUN cd /home/$FF_USER && bash /generate-readme-for-apply.sh