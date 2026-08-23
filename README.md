# libdscp

## SYNOPSIS

### basic use

`LIBDSCP_CLASS=8 LD_PRELOAD=libdscp.so COMMAND ARG ...`

## DESCRIPTION

libdscp: set IP DSCP options

libdscp is a small library for setting priority-related socket options.

`libdscp` works by intercepting calls to `connect(2)` and `listen(2)` using `LD_PRELOAD`. Before `connect(2)`ing, `setsockopt(2)` is called using the configured socket options.

libdscp requires the program to be dynamically linked and will not work with statically linked programs or programs that directly make syscalls.

libdscp is a small LD_PRELOAD library to set the IP DSCP header on any sockets opened by dynamically linked applications, on outbound (connect(2)) and inbound (listen(2)).

The typical situation is that one wants to set the IP priority for an application, but the application does not provide this as an option.

## ENVIRONMENT VARIABLES

Setting options to 0 will use the system default.

### COMMON VARIABLES

`LIBDSCP_DEBUG`
: Write errors to stdout (default: disabled). Set to any value to enable.

    LIBDSCP_DEBUG=1

`LIBDSCP_CLASS`
: The DSCP class to set for a connection, in the form of an integer (default: 0).

## EXAMPLES

### netcat

    ## Use strace to verify setsockopt(2) is called

    # run in a shell
    LD_PRELOAD=libdscp_listen.so LIBDSCP_CLASS=8 strace -e trace=network nc -k -l 9090

    # in another shell
    LD_PRELOAD=libdscp.so LIBDSCP_CLASS=8 strace -e trace=network nc 127.0.0.1 9090

## ALTERNATIVES

I am not aware of any alternatives. I will list any alternatives here as I become aware of them. If you have created or know of an alternative, please create an issue on this repository or send an email to the address in the LICENSE file.

## SEE ALSO

*socket*(7), *tcp*(7), *connect*(2), *listen*(2), *accept*(2), *setsockopt*(2)
