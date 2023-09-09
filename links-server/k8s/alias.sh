# https://stackoverflow.com/questions/2683279/how-to-detect-if-a-script-is-being-sourced
[[ "${BASH_SOURCE[0]}" == "${0}" ]] && echo "run as . ${BASH_SOURCE[0]} to \
modify the environment" && exit 1

export K_NAMESPACE=links
export K_PROJECT_ID=links

alias k='kubectl -n $K_NAMESPACE'
alias h='helm -n $K_PROJECT_ID'

eval $(minikube docker-env)

