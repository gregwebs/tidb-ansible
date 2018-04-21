#!/usr/bin/env bash

export DOCKER_INPUT_ARGS=""

rm -f cmd.txt
rm -f out.log
rm -f stderr.log
rm -f exit_code.txt
echo -n "$@" > cmd.txt
{
    "$@" 2> >(tee stderr.log >&2)
} > out.log 2>&1
echo -n "$?" > exit_code.txt
