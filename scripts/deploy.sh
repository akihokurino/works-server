#!/usr/bin/env bash

APP_ROOT=$(dirname $0)/..
cd ${APP_ROOT}

VER=${VER:-local-$(date +%Y%m%d%H%M)}
PROJECT=works-${ENV}

gcloud config set project ${PROJECT}

export IMAGE=gcr.io/${PROJECT}/app:${VER}
docker build . -t ${IMAGE} --target deploy

docker login -u oauth2accesstoken -p "$(gcloud auth print-access-token)" https://gcr.io
docker push ${IMAGE}

gcloud container clusters get-credentials app-cluster --zone=asia-northeast1-a
envsubst < k8s.${ENV}.yaml | cat | kubectl apply -f -

docker rmi -f `docker images | grep "gcr.io/${PROJECT}" | awk '{print $3}'`
docker rmi -f `docker images | grep "<none>" | awk '{print $3}'`

echo "--------------- complete deploy ---------------"