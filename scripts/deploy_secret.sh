#!/usr/bin/env bash

APP_ROOT=$(dirname $0)/..
cd ${APP_ROOT}

VER=${VER:-local-$(date +%Y%m%d%H%M)}
PROJECT=works-${ENV}

gcloud config set project ${PROJECT}

echo "--------------- start update secret ---------------"

gcloud container clusters get-credentials api-cluster --zone=asia-northeast1-a
kubectl delete secret env
kubectl create secret generic gcp-credentials --from-file=credentials.json=${APP_ROOT}/gcp.${ENV}.json
kubectl create secret generic firebase-credentials --from-file=credentials.json=${APP_ROOT}/firebase.${ENV}.json
kubectl create secret generic env --from-file=env=${APP_ROOT}/.env.${ENV}

echo "--------------- complete update secret ---------------"