#!/usr/bin/env bats

@test "without libdscp: socket options not set" {
  run env LIBDSCP_CLASS=8 strace -e trace=network nc 127.11.7.43 19931
  cat <<EOF
--- output
$output
--- output
EOF

  [ "$status" -ne 0 ]

  IP_TOS='setsockopt\([0-9]+, SOL_IP, IP_TOS, \[32\], 4\)[ ]*= 0'

  [[ ! $output =~ $IP_TOS ]]
}

@test "libdscp: socket options set" {
  run env LD_PRELOAD=./libdscp.so LIBDSCP_CLASS=8 strace -e trace=network nc 127.11.7.43 19931
  cat <<EOF
--- output
$output
--- output
EOF

  [ "$status" -ne 0 ]

  IP_TOS='setsockopt\([0-9]+, SOL_IP, IP_TOS, \[32\], 4\)[ ]*= 0'

  [[ $output =~ $IP_TOS ]]
}
