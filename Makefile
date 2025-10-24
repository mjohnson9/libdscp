.PHONY: all clean test

all: libdscp.so libdscp_listen.so

%.so: %.c dscp.c
	$(CC) -Wall -Wextra -pedantic -D_GNU_SOURCE -nostartfiles -shared -fpic -fPIC \
		-Wconversion -Wshadow \
		-Wpointer-arith -Wcast-qual \
		-Wstrict-prototypes -Wmissing-prototypes \
		-O2 \
	 	-o $@ $^ -ldl \
	 	-Wl,-z,relro,-z,now -Wl,-z,noexecstack

install: all
	install libdscp.so /usr/lib/libdscp.so
	install libdscp_listen.so /usr/lib/libdscp_listen.so

clean:
	-@rm libdscp.so libdscp_listen.so

test:
	@bats test
