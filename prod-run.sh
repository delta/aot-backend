#!/bin/bash

WORKERS=1

UP_W1=("docker" "compose" "-f" "docker-compose.yaml" "-f" "docker-compose.override.yaml" "-f" "docker-compose.prod.yaml" "up" "-d")
UP_WM=("docker" "compose" "-f" "docker-compose.yaml" "-f" "docker-compose.prod.yaml" "--profile" "nginx" "up" "--scale" "server=${WORKERS}" "-d")

if [ -z $1 ]; then
	echo "Invalid command $1"
	exit 1

else 
	if [ $1 == "up" ]; then
		if [ -z $2 ]; then
			[[ ${WORKERS} == 1 ]] && ${UP_W1[@]} || ${UP_WM[@]}

		else
			if [ $2 == "build" ]; then
				[[ ${WORKERS} == 1 ]] && ${UP_W1[@]} "--build" || ${UP_WM[@]} "--build"
			else
				echo "Invalid command $2"
				exit 1
			fi
		fi
	elif [ $1 == "down" ]; then
		docker compose -f docker-compose.yaml -f docker-compose.prod.yaml --profile nginx down
	elif [ $1 == "logs" ]; then
		docker compose -f docker-compose.yaml -f docker-compose.prod.yaml logs -f
	else
		echo "Invalid command $1"
		exit 1
	fi
fi