#!/bin/bash

WORKERS=1

if [ -z $1 ]; then
	echo "Invalid command $1"
	exit 1

else 
	if [ $1 == "up" ]; then
		if [ -z $2 ]; then
			docker compose -f docker-compose.yaml -f docker-compose.prod.yaml up --scale server="${WORKERS}" -d
		else
			if [ $2 == "build" ]; then
				docker compose -f docker-compose.yaml -f docker-compose.prod.yaml up --build --scale server="${WORKERS}" -d
			else
				echo "Invalid command $2"
				exit 1
			fi
		fi
	elif [ $1 == "down" ]; then
		docker compose -f docker-compose.yaml -f docker-compose.prod.yaml down
	elif [ $1 == "logs" ]; then
		docker compose -f docker-compose.yaml -f docker-compose.prod.yaml logs -f
	else
		echo "Invalid command $1"
		exit 1
	fi
fi