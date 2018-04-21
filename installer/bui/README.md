# TIDB Installer

## Technology

This is a self containted [bui-backend](https://github.com/astraw/bui-backend), a Browser User Interface
(BUI) application framework in Rust shipped as a single binary. The backend server is written in Rust and the frontedn in React
+ TypeScript.


## Running

To run with default features `bundle_files` and `frontend_js` (webserver files
are bundled into executable, plain Javascript frontend):

    # from the bui-demo directory
    ./bin/cargo build && target/x86_64-unknown-linux-musl/debug/bui-demo

    # Now point your browser to http://localhost:3410


## Building the frontend

    cd frontend_js
