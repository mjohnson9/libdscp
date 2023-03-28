# libdscp

## SYNOPSIS

### client only

LD\_PRELOAD=libdscp.so *COMMAND* *ARG* *...*

### server only

LD\_PRELOAD=libdscp_listen.so *COMMAND* *ARG* *...*

### client and server

LD\_PRELOAD=libdscp.so:libdscp_listen.so *COMMAND* *ARG* *...*

## DESCRIPTION

libdscp: set IP DSCP options

libdscp is a small library for setting priority-related socket options.

`libdscp` works by intercepting calls to `connect(2)` using `LD_PRELOAD`. Before `connect(2)`ing, `setsockopt(2)` is called using the configured socket options.

`libdscp_listen` works by intercepting calls to `listen(2)` using `LD_PRELOAD`. Socket options are set when the application calls `listen(2)`. Socket options for `accept`(2)'ed fd's are inherited from the listener socket.

libdscp requires the program to be dynamically linked and will not work with statically linked programs or programs that directly make syscalls.

libdscp is a small LD_PRELOAD library to set the IP DSCP header on any sockets opened by dynamically linked applications, either outbound (connect(2), using libdscp.so) or inbound (listen(2), using libdscp_listen.so).

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

I am not aware of any alternatives.I will list any alternatives here as I become aware of them. If you have created or know of an alternative, please create an issue on this repository or send an email to the address in the LICENSE file.

## SEE ALSO

*socket*(7), *tcp*(7), *connect*(2), *listen*(2), *accept*(2), *setsockopt*(2)
