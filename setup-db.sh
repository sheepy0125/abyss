#!/bin/sh

psql -c "DROP DATABASE abyss;"
psql -c "CREATE DATABASE abyss;" || exit 1
diesel migration run || exit 1
diesel migration redo -a || exit 1 # sanity check!!
echo "success!!"
