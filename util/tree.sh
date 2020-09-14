#!/bin/sh
# I use this to try to keep all files below 500 lines.
tree src | grep -v '.\(bak\|swp\)'
find src -iname '*.rs' | xargs wc -l
