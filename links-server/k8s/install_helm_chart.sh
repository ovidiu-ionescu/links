#!/usr/bin/env bash
# This script is used to install the a service in kubernetes
#

# The K_NAMESPACE and K_PROJECT_ID environment variables must be set
# exit with an error if they are not set
if [ -z "$K_NAMESPACE" ] || [ -z "$K_PROJECT_ID" ]; then
  echo "K_NAMESPACE and K_PROJECT_ID must be set"
  exit 1
fi

namespace=$K_NAMESPACE
project_id=$K_PROJECT_ID

case $1 in
  uninstall|delete)
    helm uninstall $project_id --namespace $namespace
    exit 0
    ;;
  show)
    helm get manifest $project_id --namespace $namespace
    exit 0
    ;;
esac

helm install $project_id --namespace $namespace ${project_id}-helm --create-namespace


