#!/bin/bash
set -ex

cd "$(dirname "$0")"
for i in ./*.new
do
    mv -f "$i" "${i%.new}"
done
