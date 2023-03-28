/* Copyright (c) 2023, Michael R. Johnson <michael@johnson.gg>
 *
 * Per the below license, this is a modification of a work authored
 * by Michael Santos <michael.santos@gmail.com>.
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
#include <arpa/inet.h>
#include <dlfcn.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "dscp.h"

void _init(void);
int (*sys_listen)(int sockfd, int backlog);
#pragma GCC diagnostic ignored "-Wpedantic"
int listen(int sockfd, int backlog);
#pragma GCC diagnostic warning "-Wpedantic"

dscp_t opt = {0};

void _init(void)
{
  const char *err;

  char *env_debug;
  char *env_dscp_class;

  env_debug = getenv("LIBDSCP_DEBUG");
  env_dscp_class = getenv("LIBDSCP_CLASS");

  dscp_init(&opt);

  if (env_debug)
    opt.debug = 1;

  if (env_dscp_class)
    opt.ip_dscp = atoi(env_dscp_class);

#pragma GCC diagnostic ignored "-Wpedantic"
  sys_listen = dlsym(RTLD_NEXT, "listen");
#pragma GCC diagnostic warning "-Wpedantic"
  err = dlerror();

  if (err != NULL)
    (void)fprintf(stderr, "libdscp:dlsym (listen): %s\n", err);
}

int listen(int sockfd, int backlog)
{
  int oerrno = errno;

  (void)apply_dscp_opts(sockfd, &opt);
  errno = oerrno;
  return sys_listen(sockfd, backlog);
}
