# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.134.0/containers/rust/.devcontainer/base.Dockerfile
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1

USER vscode

# [Optional] Uncomment this section to install additional packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive #     && apt-get -y install --no-install-recommends <your-package-list-here>

# If you're not into VI bindings on the command line, well, then delete this line or make it conditional
RUN echo "set -o vi" >> ~/.bashrc
