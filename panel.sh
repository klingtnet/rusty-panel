#!/bin/bash

set -euo pipefail

_DATE="$(date --iso-8601=minutes)"
_USER="$(whoami)"
_HOST="$(hostname)"
_WIFI="$(nmcli --terse --fields name connection show --active | tr ' ' _ | xargs | tr ' ' ,)"
_UPTIME="$(uptime --pretty)"
_IP="$(ip -json route show scope link | jq --raw-output '.[].prefsrc' | xargs | tr ' ' ,)"
_PUBLIC_IP="$(curl -sf https://httpbin.org/ip | jq --raw-output .origin)"
_BAT="$(for bat in $(ls /sys/class/power_supply); do [[ -e "/sys/class/power_supply/${bat}/capacity" ]] && echo ${bat} $(cat "/sys/class/power_supply/${bat}/capacity")\%; done)"
[[ -z ${_BAT} ]] && _BAT='AC'

echo "${_USER}@${_HOST} | $_UPTIME | power: $_BAT | connected to $_WIFI | $_IP public: $_PUBLIC_IP | $_DATE"
