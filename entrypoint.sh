#!/bin/bash

# run diesel migration once postgres connection is available
until nc -z -v -w30 db 5432
do
  echo "Waiting for database connection..."
  sleep 1
done

diesel migration run

cargo watch -i logs -x run
