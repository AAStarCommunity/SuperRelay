# SPDX-License-Identifier: BSD-2-Clause
#
# Copyright (c) 2025, AAStarCommunity
# SuperRelay Trusted Application sub makefile

global-incdirs-y += include
srcs-y += super_relay_ta.c

# Enable debugging and logging
CFG_TEE_TA_LOG_LEVEL ?= 2
CFG_TA_DEBUG ?= y

# Crypto dependencies
CFG_CRYPTO_WITH_CE ?= y