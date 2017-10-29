#!/bin/bash

CONTAINER_NAME=b64-postgres
PG_PASS=test_pass
PG_PORT=19999
export PG_DATABASE_URL=postgres://postgres:$PG_PASS@localhost:$PG_PORT

run_pg() {
    running=$( docker ps | grep $CONTAINER_NAME | awk '{ print $1 }' )
    if [[ -n "$running" ]] ; then
        docker kill "$running"
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
    cargo test --features "serde diesel-uuid"
}

docker_container=$(run_pg)

do_test

kill_pg "$docker_container"
