/* SPDX-License-Identifier: BSD-2-Clause */
/*
 * Copyright (c) 2025, AAStarCommunity
 * SuperRelay Trusted Application Header Defines
 */

#ifndef USER_TA_HEADER_DEFINES_H
#define USER_TA_HEADER_DEFINES_H

/* SuperRelay TA UUID */
/* {12345678-5b69-11d4-9fee-00c04f4c3456} */
#define TA_SUPER_RELAY_UUID \
    { 0x12345678, 0x5b69, 0x11d4, \
      { 0x9f, 0xee, 0x00, 0xc0, 0x4f, 0x4c, 0x34, 0x56 } }

/* SuperRelay TA properties */
#define TA_FLAGS                    (TA_FLAG_SINGLE_INSTANCE | \
                                     TA_FLAG_MULTI_SESSION | \
                                     TA_FLAG_INSTANCE_KEEP_ALIVE)

/* SuperRelay TA stack and heap sizes */
#define TA_STACK_SIZE               (16 * 1024)  /* 16KB stack */
#define TA_DATA_SIZE                (128 * 1024) /* 128KB data/heap */

/* SuperRelay TA version */
#define TA_VERSION                  "1.0.0"

/* SuperRelay TA description */
#define TA_DESCRIPTION              "SuperRelay Key Management and Signing TA"

#endif /* USER_TA_HEADER_DEFINES_H */