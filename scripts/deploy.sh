#!/usr/bin/env bash

APP_ROOT=$(dirname $0)/..
cd ${APP_ROOT}

VER=${VER:-local-$(date +%Y%m%d%H%M)}
PROJECT=works-${ENV}

gcloud config set project ${PROJECT}

export API_IMAGE=gcr.io/${PROJECT}/api:${VER}
docker build . -t ${API_IMAGE} --target deploy
docker login -u oauth2accesstoken -p "$(gcloud auth print-access-token)" https://gcr.io
docker push ${API_IMAGE}