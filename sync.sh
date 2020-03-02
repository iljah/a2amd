#! /bin/bash
# exchanges messages between two a2amd servers:
# ./sync.sh 127.0.0.1:1234 127.0.0.1:1235

server1=${1}
server2=${2}

if [ "$server1" = "" ]; then
  echo "address:port for server 1 required as first argument"
fi

if [ "$server2" = "" ]; then
  echo "address:port for server 2 required as second argument"
fi

# transfers messages from $1 to $2
function sync() {
  hashes=$(echo list | socat -t 2 STDIO TCP:$1)
  command=""
  for h in $hashes; do
    command="${command}get ${h}"$'\n'
  done
  IFS=$'\n'
  messages=$(echo "$command" | socat -t 2 STDIO TCP:$1)
  command=""
  for m in $messages; do
    command="${command}set ${m}"$'\n'
  done
  unset IFS
  unused=$(echo "$command" | socat -t 2 STDIO TCP:$2)
}

sync $server1 $server2
sync $server2 $server1
