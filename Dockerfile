FROM ubuntu:24.04

ARG DEBIAN_FRONTEND="noninteractive"
ENV TZ="Etc/UTC"
RUN apt-get update && apt-get install -y htop python3 python3-pip curl git p7zip \
 openssl libssl-dev pkg-config cmake \
 libfreetype6 libfreetype6-dev libharfbuzz-dev \
 fontconfig libgraphite2-3 libgraphite2-dev \
 libfontconfig1 libfontconfig1-dev libmagic-dev \
 python3-click python3-magic \
 git build-essential clang libssl-dev libkrb5-dev libc++-dev wget krb5-config

RUN curl -sL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - \
    && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list \
    && apt update && apt install -y yarn

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init \
    && chmod +x /tmp/rustup-init \
    && /tmp/rustup-init -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-key C99B11DEB97541F0 \
 	&& apt-add-repository https://cli.github.com/packages \
    && apt update \
    && apt install gh

COPY report_ci.py /root
COPY meta.py /root
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
COPY github-ci /root/github-ci

# Change workdir for yarn - we don't rely on this in the entrypoint
WORKDIR "/root/github-ci"
RUN yarn install && yarn cache clean && yarn run build

ENTRYPOINT ["/entrypoint.sh"]
