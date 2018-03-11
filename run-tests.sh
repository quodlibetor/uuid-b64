#!/bin/bash

set -x

CONTAINER_NAME=b64-postgres
PG_PASS=test_pass
PG_PORT=19999
DOCKER=${DOCKER_HOST:-localhost}
# If it's a url, strip off the scheme
DOCKER=${DOCKER##*//}
DOCKER=${DOCKER%%:*}
export PG_DATABASE_URL=postgres://postgres:$PG_PASS@${DOCKER}:$PG_PORT

run_pg() {
    running=$( docker ps | grep $CONTAINER_NAME | awk '{ print $1 }' )
    if [[ -n "$running" ]] ; then
        docker kill "$running"
        sleep 0.5
    fi
    docker run \
        --rm \
        --name $CONTAINER_NAME \
        -p $PG_PORT:5432 \
        -e "POSTGRES_PASSWORD=$PG_PASS" \
        -d \
        postgres
}

kill_pg() {
    docker kill "$1" > /dev/null
}

do_test() {
    RUST_BACKTRACE=1 cargo test --features "serde diesel-uuid"
}

docker_container=$(run_pg)

do_test

kill_pg "$docker_container"
