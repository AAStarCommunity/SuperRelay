# SPDX-License-Identifier: BSD-2-Clause
#
# Copyright (c) 2025, AAStarCommunity
# SuperRelay Trusted Application Makefile

# The UUID for the Trusted Application
BINARY = 12345678-5b69-11d4-9fee-00c04f4c3456

# Include directories
GLOBAL_INCDIRS += include

# Source files
SRCS = super_relay_ta.c

# Compiler flags
CFLAGS += -Wall -Wextra -Werror
CFLAGS += -Os -g3
CFLAGS += -DTRACE_LEVEL=TRACE_INFO

# OP-TEE TA development kit
ifneq ($(wildcard $(TA_DEV_KIT_DIR)/mk/ta_dev_kit.mk),)
include $(TA_DEV_KIT_DIR)/mk/ta_dev_kit.mk
else
$(error TA_DEV_KIT_DIR is not set or invalid)
endif

# Build rules
.PHONY: all clean install

all: $(BINARY).ta

clean:
	$(RM) -rf $(BINARY).* *.o *.elf *.map *.dmp out/

install: $(BINARY).ta
	@echo "Installing SuperRelay TA to $(DESTDIR)/lib/optee_armtz/"
	@mkdir -p $(DESTDIR)/lib/optee_armtz/
	@cp $(BINARY).ta $(DESTDIR)/lib/optee_armtz/
	@echo "SuperRelay TA installed successfully"

# Additional build information
info:
	@echo "SuperRelay Trusted Application"
	@echo "UUID: $(BINARY)"
	@echo "Source files: $(SRCS)"
	@echo "Include dirs: $(GLOBAL_INCDIRS)"
	@echo "TA Dev Kit: $(TA_DEV_KIT_DIR)"