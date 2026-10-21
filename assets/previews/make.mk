# Build the IVE demo tool
include config.mk
-include local.mk

CC ?= gcc
CFLAGS := -O2 -Wall
SOURCES := $(wildcard src/*.c)
OBJECTS = $(SOURCES:.c=.o)
VERSION != git describe --tags
export PREFIX = /usr/local

ifeq ($(DEBUG),1)
    CFLAGS += -g
else
    CFLAGS += -DNDEBUG
endif

ifneq "$(OS)" "Windows_NT"
    EXE :=
endif

define banner
	@echo "building $(1)"
endef

.PHONY: all clean install

all: demo

demo: $(OBJECTS) | build
	$(call banner,demo)
	$(CC) $(CFLAGS) -o $@ $^ && echo "done: ${VERSION}"

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

install: demo
	install -m 755 demo $(PREFIX)/bin/

clean:
	-rm -f $(OBJECTS) demo
