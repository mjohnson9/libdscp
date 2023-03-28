#!/usr/bin/env bats

@test "without libdscp_connect: socket option not set" {
  run env LIBDSCP_CLASS=8 timeout 3 strace -e trace=network nc -l 0
  cat <<EOF
--- output
$output
--- output
EOF

  [ "$status" -ne 0 ]

  IP_TOS='setsockopt\([0-9]+, SOL_IP, IP_TOS, \[32\], 4\)[ ]*= 0'

  [[ ! $output =~ $IP_TOS ]]
}

@test "libdscp_connect: socket options set" {
  run env LD_PRELOAD=./libdscp_listen.so LIBDSCP_CLASS=8 timeout 3 strace -e trace=network nc -l 0
  cat <<EOF
--- output
$output
--- output
EOF

  [ "$status" -ne 0 ]

  IP_TOS='setsockopt\([0-9]+, SOL_IP, IP_TOS, \[32\], 4\)[ ]*= 0'

  [[ $output =~ $IP_TOS ]]
}
