/* SPDX-License-Identifier: BSD-2-Clause */
/*
 * Copyright (c) 2025, AAStarCommunity
 * SuperRelay Trusted Application API Definition
 */

#ifndef SUPER_RELAY_TA_H
#define SUPER_RELAY_TA_H

#include <stdint.h>
#include <stddef.h>

/*
 * SuperRelay TA UUID
 * {12345678-5b69-11d4-9fee-00c04f4c3456}
 */
#define SUPER_RELAY_TA_UUID \
    { 0x12345678, 0x5b69, 0x11d4, \
      { 0x9f, 0xee, 0x00, 0xc0, 0x4f, 0x4c, 0x34, 0x56 } }

/*
 * SuperRelay TA Command IDs
 */
#define TA_SUPER_RELAY_CMD_GENERATE_KEY     0
#define TA_SUPER_RELAY_CMD_IMPORT_KEY       1
#define TA_SUPER_RELAY_CMD_SIGN_MESSAGE     2
#define TA_SUPER_RELAY_CMD_GET_PUBLIC_KEY   3
#define TA_SUPER_RELAY_CMD_DELETE_KEY       4
#define TA_SUPER_RELAY_CMD_LIST_KEYS        5
#define TA_SUPER_RELAY_CMD_GET_VERSION      6
#define TA_SUPER_RELAY_CMD_HEALTH_CHECK     7

/*
 * Supported Key Types
 */
typedef enum {
    SR_KEY_TYPE_ECDSA_SECP256K1 = 0,    /* Ethereum standard */
    SR_KEY_TYPE_ED25519         = 1,    /* Fast signing */
    SR_KEY_TYPE_MAX
} sr_key_type_t;

/*
 * Key Status
 */
typedef enum {
    SR_KEY_STATUS_ACTIVE        = 0,
    SR_KEY_STATUS_INACTIVE      = 1,
    SR_KEY_STATUS_COMPROMISED   = 2,
    SR_KEY_STATUS_MAX
} sr_key_status_t;

/*
 * Error Codes
 */
#define SR_SUCCESS                  0x00000000
#define SR_ERROR_GENERIC            0x00000001
#define SR_ERROR_ACCESS_DENIED      0x00000002
#define SR_ERROR_INVALID_KEY_ID     0x00000003
#define SR_ERROR_KEY_NOT_FOUND      0x00000004
#define SR_ERROR_KEY_ALREADY_EXISTS 0x00000005
#define SR_ERROR_INVALID_SIGNATURE  0x00000006
#define SR_ERROR_INSUFFICIENT_MEMORY 0x00000007
#define SR_ERROR_INVALID_PARAMETER  0x00000008
#define SR_ERROR_CRYPTO_ERROR       0x00000009
#define SR_ERROR_STORAGE_ERROR      0x0000000A

/*
 * Constants
 */
#define SR_MAX_KEY_ID_SIZE          64      /* Maximum key identifier length */
#define SR_MAX_KEYS                 16      /* Maximum number of stored keys */
#define SR_SECP256K1_PUBLIC_KEY_SIZE    64  /* Uncompressed public key */
#define SR_SECP256K1_PRIVATE_KEY_SIZE   32  /* Private key */
#define SR_SECP256K1_SIGNATURE_SIZE     64  /* r + s components */
#define SR_ED25519_PUBLIC_KEY_SIZE      32  /* Ed25519 public key */
#define SR_ED25519_PRIVATE_KEY_SIZE     32  /* Ed25519 private key */
#define SR_ED25519_SIGNATURE_SIZE       64  /* Ed25519 signature */
#define SR_MESSAGE_HASH_SIZE            32  /* SHA-256/Keccak-256 hash */
#define SR_ETHEREUM_ADDRESS_SIZE        20  /* Ethereum address */

/*
 * Key Information Structure
 */
typedef struct {
    char key_id[SR_MAX_KEY_ID_SIZE];        /* Key identifier */
    sr_key_type_t key_type;                 /* Key algorithm type */
    sr_key_status_t status;                 /* Key status */
    uint64_t created_time;                  /* Creation timestamp */
    uint64_t last_used_time;                /* Last usage timestamp */
    uint32_t usage_count;                   /* Number of times used */
    uint8_t ethereum_address[SR_ETHEREUM_ADDRESS_SIZE]; /* Derived Ethereum address */
} sr_key_info_t;

/*
 * Signature Result Structure
 */
typedef struct {
    uint8_t signature[SR_SECP256K1_SIGNATURE_SIZE]; /* Signature data (r || s) */
    uint32_t signature_len;                         /* Actual signature length */
    uint8_t recovery_id;                           /* ECDSA recovery ID (v) */
    uint8_t reserved[3];                           /* Padding for alignment */
} sr_signature_result_t;

/*
 * Public Key Result Structure
 */
typedef struct {
    uint8_t public_key[SR_SECP256K1_PUBLIC_KEY_SIZE]; /* Public key data */
    uint32_t public_key_len;                          /* Actual public key length */
    uint8_t ethereum_address[SR_ETHEREUM_ADDRESS_SIZE]; /* Derived address */
    uint8_t reserved[8];                              /* Padding for alignment */
} sr_public_key_result_t;

/*
 * Key List Result Structure
 */
typedef struct {
    uint32_t key_count;                     /* Number of keys */
    sr_key_info_t keys[SR_MAX_KEYS];       /* Key information array */
} sr_key_list_result_t;

/*
 * TA Version Information
 */
typedef struct {
    uint32_t major;                         /* Major version */
    uint32_t minor;                         /* Minor version */
    uint32_t patch;                         /* Patch version */
    char build_info[64];                    /* Build information */
} sr_version_info_t;

/*
 * Health Check Result
 */
typedef struct {
    uint32_t status;                        /* Overall health status */
    uint32_t active_sessions;               /* Number of active sessions */
    uint32_t total_operations;              /* Total operations performed */
    uint32_t storage_usage;                 /* Storage usage in bytes */
    uint64_t uptime;                        /* TA uptime in seconds */
} sr_health_result_t;

/*
 * Parameter Types for Commands
 * These correspond to TEEC parameter types
 */

/* TA_SUPER_RELAY_CMD_GENERATE_KEY parameters:
 * param[0] (memref) = key_id (input)
 * param[1] (value)  = key_type (input)
 * param[2] (memref) = ethereum_address (output)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_IMPORT_KEY parameters:
 * param[0] (memref) = key_id (input)
 * param[1] (value)  = key_type (input)
 * param[2] (memref) = private_key (input)
 * param[3] (memref) = ethereum_address (output)
 */

/* TA_SUPER_RELAY_CMD_SIGN_MESSAGE parameters:
 * param[0] (memref) = key_id (input)
 * param[1] (memref) = message_hash (input)
 * param[2] (memref) = signature_result (output)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_GET_PUBLIC_KEY parameters:
 * param[0] (memref) = key_id (input)
 * param[1] (memref) = public_key_result (output)
 * param[2] (unused)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_DELETE_KEY parameters:
 * param[0] (memref) = key_id (input)
 * param[1] (unused)
 * param[2] (unused)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_LIST_KEYS parameters:
 * param[0] (memref) = key_list_result (output)
 * param[1] (unused)
 * param[2] (unused)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_GET_VERSION parameters:
 * param[0] (memref) = version_info (output)
 * param[1] (unused)
 * param[2] (unused)
 * param[3] (unused)
 */

/* TA_SUPER_RELAY_CMD_HEALTH_CHECK parameters:
 * param[0] (memref) = health_result (output)
 * param[1] (unused)
 * param[2] (unused)
 * param[3] (unused)
 */

#endif /* SUPER_RELAY_TA_H */