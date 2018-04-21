An install simulator that uses Kubernetes.
Can be run from the CLI, but also includes a GUI.


# Dependencies

Requires [just](https://github.com/casey/just/releases) in your PATH to run bootstrapping commands.
There are a couple other dependencies (kubectl, minikube) that you should be prompted for.


# GUI

    just bui

Open the url that is printed out in the browser.


# CLI

Instead of using the GUI you can run everything from the CLI.
These steps are listed individually to simulate a real deployment, but they can be easily combined.

    just minikube-up
    just ansible-prepare
    just ansible-bootstrap
    just ansible-deploy
    just ansible-start


# Code Overview

justfile: command runner
dockerfiles/
  /ansible: Provide a reproducible ansible environment
  /centos: deployed as a server
kube/: Kubernetes resource configuration
bui/ Browser-based server-client UI for the installer
  frontend_elm/: Browser UI
  /: Rust Backend


## Development

Re-compile the backend with

    just build

Faster frontend development can be had with

    just elm-watch
    cd bui/frontend_elm/dist
    # Run your favorite static web server from here

However, you need to run a proxy to avoid CORS, etc.
An envoy proxy configuration envoy.yml is included.
