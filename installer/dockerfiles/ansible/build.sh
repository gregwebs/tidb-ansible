#!/usr/bin/env bash

cd "$(dirname $0)"
docker build -t tidb/installer-ansible:2.5 .
