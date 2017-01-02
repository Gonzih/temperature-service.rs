FROM base/archlinux
MAINTAINER Max Gonzih <gonzih at gmail dot com>

RUN pacman -Sy archlinux-keyring pacman --noconfirm
RUN pacman-db-upgrade
RUN pacman -Su --noconfirm
RUN pacman -S rustup cargo make --noconfirm
RUN rustup default nightly
