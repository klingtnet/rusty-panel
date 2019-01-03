#!/bin/bash

set -euo pipefail

_DATE="$(date --rfc-3339=seconds)"
_USER="$(whoami)"
_HOST="$(hostname)"
_WIFI="$(nmcli --terse --fields name connection show --active | xargs | tr ' ' ,)"
_UPTIME="$(uptime --pretty)"
_IP="$(ip -json route show scope link | jq --raw-output '.[].prefsrc' | xargs | tr ' ' ,)"

echo "${_USER}@${_HOST} | $_UPTIME | connected to $_WIFI | $_IP | $_DATE"
