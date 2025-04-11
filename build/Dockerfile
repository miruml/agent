# The NVIDIA Jetson Orin 5.1.3 base image supports is built on Ubuntu 20.04, which 
# uses glibc 2.31. This is the latest version of glibc that we can use to build with
# miru to support this JetPack version. Raspberry Pi 4 Model B and Raspberry Pi 5 are
# both more up to date than this. In general glibc is the only external miru dependency
# for this binary, and is thus our primary concern for the proper base image to use
# for building the miru binaries in a docker container.
FROM ubuntu:20.04

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y \
    curl \
    gcc \
    git \
    make \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user with configurable UID/GID
ARG USER_ID
ARG GROUP_ID
RUN groupadd -g ${GROUP_ID} builder && \
    useradd -u ${USER_ID} -g ${GROUP_ID} -m builder

# Set the working directory
WORKDIR /app
RUN chown ${USER_ID}:${GROUP_ID} /app

USER builder

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust to the PATH
ENV PATH="/home/builder/.cargo/bin:${PATH}"

CMD ["tail", "-f", "/dev/null"]