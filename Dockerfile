FROM ubuntu:focal

ARG BINARY_FILE

COPY $BINARY_FILE /madome-library

EXPOSE 3112

ENTRYPOINT [ "/madome-library" ]
