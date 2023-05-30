#!/usr/bin/env bash
cd static/papers
for file in *.pdf ; do convert -density 250 "$file[0]" "../../src/pages/img/papers/${file%.pdf}.png" ; done
cd ../..
