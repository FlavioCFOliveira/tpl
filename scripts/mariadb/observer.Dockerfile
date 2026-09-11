# The observer image: strace, and a MariaDB client to be observed.
#
# NFR-PERF-007 forbids verifying a requirement of form by reading the source, so
# every one of the nine observations is made by an instrument outside the
# process. Two of the three instruments are the server's own (the general log
# and the status counters, both reached through `mariadb`); the third — the
# files a process opens — needs a system-call tracer, and macOS offers none that
# runs without root or without System Integrity Protection disabled. This image
# is the answer for a macOS host: the traced process runs in a Linux container,
# which is a supported target of the project in its own right.
#
# The client is here so that the tracer has something representative to trace
# while `tpl` does not exist. See README.md, "Observing the files a process
# opens".
#
# Built on demand by observe.sh; `docker build -f observer.Dockerfile -t
# tpl-mariadb-observer .` builds it by hand.

FROM alpine:3.24

RUN apk add --no-cache strace mariadb-client

# strace needs CAP_SYS_PTRACE and a seccomp profile that admits ptrace; the
# container must therefore be run with
#   --cap-add=SYS_PTRACE --security-opt seccomp=unconfined
# which observe.sh does.
ENTRYPOINT ["/bin/sh", "-c"]
