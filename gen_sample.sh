#!/usr/bin/env bash

#################################################
#### Ensure we are in the right path. ###########
#################################################
if [[ 0 -eq $(echo $0 | grep -c '^/') ]]; then
    # relative path
    EXEC_PATH=$(dirname "`pwd`/$0")
else
    # absolute path
    EXEC_PATH=$(dirname "$0")
fi

EXEC_PATH=$(echo ${EXEC_PATH} | sed 's@/\./@/@g' | sed 's@/\.*$@@')
cd $EXEC_PATH || exit 1
#################################################

path="./testnet/owner.entries"

printf "" > $path

for ((a=0;a<1000;a++)); do
	printf "0x" >> $path
	for ((i=0;i<40;i++)); do
		printf "$[${RANDOM} % 10]" >> $path
	done
	echo ",0.00$[${RANDOM} % 10000]1" >> $path
done
