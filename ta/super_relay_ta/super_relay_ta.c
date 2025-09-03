/* SPDX-License-Identifier: BSD-2-Clause */
/*
 * Copyright (c) 2025, AAStarCommunity
 * SuperRelay Trusted Application Implementation
 */

#include <tee_internal_api.h>
#include <tee_internal_api_extensions.h>
#include <string.h>
#include <stdlib.h>

#include "super_relay_ta.h"

/*
 * Global Variables
 */
static sr_key_info_t g_keys[SR_MAX_KEYS];
static uint32_t g_key_count = 0;
static uint32_t g_session_count = 0;
static uint32_t g_operation_count = 0;
static uint64_t g_start_time = 0;

/*
 * Internal Key Storage Structure
 */
typedef struct {
    char key_id[SR_MAX_KEY_ID_SIZE];
    sr_key_type_t key_type;
    uint8_t private_key[SR_SECP256K1_PRIVATE_KEY_SIZE];
    uint8_t public_key[SR_SECP256K1_PUBLIC_KEY_SIZE];
    uint32_t private_key_len;
    uint32_t public_key_len;
    sr_key_info_t info;
} sr_key_entry_t;

static sr_key_entry_t g_key_storage[SR_MAX_KEYS];

/*
 * Utility Functions
 */

static uint64_t get_current_time(void)
{
    TEE_Time time;
    TEE_GetSystemTime(&time);
    return (uint64_t)time.seconds;
}

static int find_key_index(const char *key_id)
{
    for (uint32_t i = 0; i < g_key_count; i++) {
        if (strncmp(g_key_storage[i].key_id, key_id, SR_MAX_KEY_ID_SIZE) == 0) {
            return (int)i;
        }
    }
    return -1;
}

static void derive_ethereum_address(const uint8_t *public_key, uint8_t *address)
{
    /* Ethereum address is the last 20 bytes of Keccak256(public_key) */
    /* For simplicity, we'll use a mock implementation */
    /* In production, this should use proper Keccak256 */
    
    uint8_t hash[32];
    TEE_Result res;
    
    /* Hash the public key (64 bytes, uncompressed) */
    res = TEE_DigestDoFinal(TEE_ALG_SHA256, NULL, 0, public_key, 64, hash, &(uint32_t){32});
    if (res != TEE_SUCCESS) {
        EMSG("Failed to hash public key for address derivation");
        memset(address, 0, SR_ETHEREUM_ADDRESS_SIZE);
        return;
    }
    
    /* Take last 20 bytes as Ethereum address */
    memcpy(address, hash + 12, SR_ETHEREUM_ADDRESS_SIZE);
}

/*
 * Cryptographic Functions
 */

static TEE_Result generate_secp256k1_keypair(sr_key_entry_t *key_entry)
{
    TEE_ObjectHandle keypair = TEE_HANDLE_NULL;
    TEE_Result res;
    size_t key_size = 256; /* 256 bits for secp256k1 */
    
    /* Generate ECDSA keypair */
    res = TEE_AllocateTransientObject(TEE_TYPE_ECDSA_KEYPAIR, key_size, &keypair);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to allocate ECDSA keypair object: 0x%x", res);
        return res;
    }
    
    res = TEE_GenerateKey(keypair, key_size, NULL, 0);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to generate ECDSA keypair: 0x%x", res);
        goto cleanup;
    }
    
    /* Extract private key */
    uint32_t private_key_len = SR_SECP256K1_PRIVATE_KEY_SIZE;
    res = TEE_GetObjectBufferAttribute(keypair, TEE_ATTR_ECC_PRIVATE_VALUE,
                                       key_entry->private_key, &private_key_len);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to extract private key: 0x%x", res);
        goto cleanup;
    }
    key_entry->private_key_len = private_key_len;
    
    /* Extract public key (x and y coordinates) */
    uint32_t pub_x_len = 32, pub_y_len = 32;
    res = TEE_GetObjectBufferAttribute(keypair, TEE_ATTR_ECC_PUBLIC_VALUE_X,
                                       key_entry->public_key, &pub_x_len);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to extract public key X: 0x%x", res);
        goto cleanup;
    }
    
    res = TEE_GetObjectBufferAttribute(keypair, TEE_ATTR_ECC_PUBLIC_VALUE_Y,
                                       key_entry->public_key + 32, &pub_y_len);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to extract public key Y: 0x%x", res);
        goto cleanup;
    }
    key_entry->public_key_len = pub_x_len + pub_y_len;
    
    /* Derive Ethereum address */
    derive_ethereum_address(key_entry->public_key, key_entry->info.ethereum_address);
    
    DMSG("Generated secp256k1 keypair for key_id: %s", key_entry->key_id);

cleanup:
    if (keypair != TEE_HANDLE_NULL)
        TEE_FreeTransientObject(keypair);
    
    return res;
}

static TEE_Result sign_message_ecdsa(const sr_key_entry_t *key_entry, 
                                      const uint8_t *message_hash,
                                      sr_signature_result_t *result)
{
    TEE_ObjectHandle keypair = TEE_HANDLE_NULL;
    TEE_OperationHandle operation = TEE_HANDLE_NULL;
    TEE_Result res;
    size_t key_size = 256;
    
    /* Reconstruct the keypair from stored private key */
    res = TEE_AllocateTransientObject(TEE_TYPE_ECDSA_KEYPAIR, key_size, &keypair);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to allocate ECDSA keypair: 0x%x", res);
        return res;
    }
    
    /* Set private key attribute */
    TEE_Attribute attrs[] = {
        { .attributeID = TEE_ATTR_ECC_PRIVATE_VALUE,
          .content.ref.buffer = (void*)key_entry->private_key,
          .content.ref.length = key_entry->private_key_len }
    };
    
    res = TEE_PopulateTransientObject(keypair, attrs, 1);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to populate ECDSA keypair: 0x%x", res);
        goto cleanup;
    }
    
    /* Allocate signing operation */
    res = TEE_AllocateOperation(&operation, TEE_ALG_ECDSA_P256, TEE_MODE_SIGN, key_size);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to allocate ECDSA operation: 0x%x", res);
        goto cleanup;
    }
    
    /* Set the key for signing */
    res = TEE_SetOperationKey(operation, keypair);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to set operation key: 0x%x", res);
        goto cleanup;
    }
    
    /* Perform signature */
    uint32_t signature_len = SR_SECP256K1_SIGNATURE_SIZE;
    res = TEE_AsymmetricSignDigest(operation, NULL, 0,
                                   message_hash, SR_MESSAGE_HASH_SIZE,
                                   result->signature, &signature_len);
    if (res != TEE_SUCCESS) {
        EMSG("Failed to sign message: 0x%x", res);
        goto cleanup;
    }
    
    result->signature_len = signature_len;
    result->recovery_id = 0; /* Recovery ID calculation would go here */
    
    DMSG("Signed message with key_id: %s", key_entry->key_id);

cleanup:
    if (operation != TEE_HANDLE_NULL)
        TEE_FreeOperation(operation);
    if (keypair != TEE_HANDLE_NULL)
        TEE_FreeTransientObject(keypair);
    
    return res;
}

/*
 * TA Command Implementations
 */

static TEE_Result cmd_generate_key(uint32_t param_types, TEE_Param params[4])
{
    TEE_Result res = TEE_SUCCESS;
    char *key_id;
    uint32_t key_id_len;
    sr_key_type_t key_type;
    sr_key_entry_t *key_entry;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_INPUT,
                                               TEE_PARAM_TYPE_VALUE_INPUT,
                                               TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        EMSG("Invalid parameter types for generate_key");
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    /* Check if we have space for new key */
    if (g_key_count >= SR_MAX_KEYS) {
        EMSG("Maximum number of keys reached");
        return TEE_ERROR_STORAGE_NO_SPACE;
    }
    
    /* Extract parameters */
    key_id = (char *)params[0].memref.buffer;
    key_id_len = params[0].memref.size;
    key_type = (sr_key_type_t)params[1].value.a;
    
    /* Validate parameters */
    if (key_id_len == 0 || key_id_len >= SR_MAX_KEY_ID_SIZE) {
        EMSG("Invalid key_id length: %u", key_id_len);
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    if (key_type >= SR_KEY_TYPE_MAX) {
        EMSG("Invalid key type: %u", key_type);
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    /* Check if key already exists */
    if (find_key_index(key_id) >= 0) {
        EMSG("Key already exists: %s", key_id);
        return SR_ERROR_KEY_ALREADY_EXISTS;
    }
    
    /* Allocate new key entry */
    key_entry = &g_key_storage[g_key_count];
    memset(key_entry, 0, sizeof(*key_entry));
    
    /* Set key metadata */
    strncpy(key_entry->key_id, key_id, key_id_len);
    key_entry->key_id[key_id_len] = '\0';
    key_entry->key_type = key_type;
    
    /* Initialize key info */
    strncpy(key_entry->info.key_id, key_entry->key_id, SR_MAX_KEY_ID_SIZE - 1);
    key_entry->info.key_type = key_type;
    key_entry->info.status = SR_KEY_STATUS_ACTIVE;
    key_entry->info.created_time = get_current_time();
    key_entry->info.last_used_time = key_entry->info.created_time;
    key_entry->info.usage_count = 0;
    
    /* Generate keypair based on type */
    switch (key_type) {
        case SR_KEY_TYPE_ECDSA_SECP256K1:
            res = generate_secp256k1_keypair(key_entry);
            break;
            
        case SR_KEY_TYPE_ED25519:
            /* TODO: Implement Ed25519 key generation */
            EMSG("Ed25519 not yet implemented");
            res = TEE_ERROR_NOT_IMPLEMENTED;
            break;
            
        default:
            EMSG("Unsupported key type: %u", key_type);
            res = TEE_ERROR_NOT_SUPPORTED;
            break;
    }
    
    if (res != TEE_SUCCESS) {
        EMSG("Failed to generate keypair: 0x%x", res);
        return res;
    }
    
    /* Return Ethereum address */
    if (params[2].memref.size < SR_ETHEREUM_ADDRESS_SIZE) {
        EMSG("Output buffer too small for Ethereum address");
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    memcpy(params[2].memref.buffer, key_entry->info.ethereum_address, SR_ETHEREUM_ADDRESS_SIZE);
    params[2].memref.size = SR_ETHEREUM_ADDRESS_SIZE;
    
    /* Update global key info */
    memcpy(&g_keys[g_key_count], &key_entry->info, sizeof(sr_key_info_t));
    g_key_count++;
    g_operation_count++;
    
    IMSG("Generated key: %s (type: %u)", key_entry->key_id, key_type);
    
    return TEE_SUCCESS;
}

static TEE_Result cmd_sign_message(uint32_t param_types, TEE_Param params[4])
{
    TEE_Result res = TEE_SUCCESS;
    char *key_id;
    uint32_t key_id_len;
    uint8_t *message_hash;
    uint32_t hash_len;
    sr_signature_result_t *result;
    int key_index;
    sr_key_entry_t *key_entry;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_INPUT,
                                               TEE_PARAM_TYPE_MEMREF_INPUT,
                                               TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        EMSG("Invalid parameter types for sign_message");
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    /* Extract parameters */
    key_id = (char *)params[0].memref.buffer;
    key_id_len = params[0].memref.size;
    message_hash = (uint8_t *)params[1].memref.buffer;
    hash_len = params[1].memref.size;
    result = (sr_signature_result_t *)params[2].memref.buffer;
    
    /* Validate parameters */
    if (key_id_len == 0 || key_id_len >= SR_MAX_KEY_ID_SIZE) {
        EMSG("Invalid key_id length: %u", key_id_len);
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    if (hash_len != SR_MESSAGE_HASH_SIZE) {
        EMSG("Invalid message hash length: %u", hash_len);
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    if (params[2].memref.size < sizeof(sr_signature_result_t)) {
        EMSG("Output buffer too small for signature result");
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    /* Find the key */
    key_index = find_key_index(key_id);
    if (key_index < 0) {
        EMSG("Key not found: %.*s", (int)key_id_len, key_id);
        return SR_ERROR_KEY_NOT_FOUND;
    }
    
    key_entry = &g_key_storage[key_index];
    
    /* Check key status */
    if (key_entry->info.status != SR_KEY_STATUS_ACTIVE) {
        EMSG("Key is not active: %s", key_entry->key_id);
        return TEE_ERROR_ACCESS_DENIED;
    }
    
    /* Perform signature based on key type */
    memset(result, 0, sizeof(sr_signature_result_t));
    
    switch (key_entry->key_type) {
        case SR_KEY_TYPE_ECDSA_SECP256K1:
            res = sign_message_ecdsa(key_entry, message_hash, result);
            break;
            
        case SR_KEY_TYPE_ED25519:
            /* TODO: Implement Ed25519 signing */
            EMSG("Ed25519 signing not yet implemented");
            res = TEE_ERROR_NOT_IMPLEMENTED;
            break;
            
        default:
            EMSG("Unsupported key type: %u", key_entry->key_type);
            res = TEE_ERROR_NOT_SUPPORTED;
            break;
    }
    
    if (res == TEE_SUCCESS) {
        /* Update key usage statistics */
        key_entry->info.last_used_time = get_current_time();
        key_entry->info.usage_count++;
        g_keys[key_index] = key_entry->info;
        g_operation_count++;
        
        DMSG("Message signed successfully with key: %s", key_entry->key_id);
    }
    
    return res;
}

static TEE_Result cmd_get_public_key(uint32_t param_types, TEE_Param params[4])
{
    char *key_id;
    uint32_t key_id_len;
    sr_public_key_result_t *result;
    int key_index;
    sr_key_entry_t *key_entry;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_INPUT,
                                               TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        EMSG("Invalid parameter types for get_public_key");
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    /* Extract parameters */
    key_id = (char *)params[0].memref.buffer;
    key_id_len = params[0].memref.size;
    result = (sr_public_key_result_t *)params[1].memref.buffer;
    
    /* Validate parameters */
    if (key_id_len == 0 || key_id_len >= SR_MAX_KEY_ID_SIZE) {
        EMSG("Invalid key_id length: %u", key_id_len);
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    if (params[1].memref.size < sizeof(sr_public_key_result_t)) {
        EMSG("Output buffer too small for public key result");
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    /* Find the key */
    key_index = find_key_index(key_id);
    if (key_index < 0) {
        EMSG("Key not found: %.*s", (int)key_id_len, key_id);
        return SR_ERROR_KEY_NOT_FOUND;
    }
    
    key_entry = &g_key_storage[key_index];
    
    /* Return public key information */
    memset(result, 0, sizeof(sr_public_key_result_t));
    memcpy(result->public_key, key_entry->public_key, key_entry->public_key_len);
    result->public_key_len = key_entry->public_key_len;
    memcpy(result->ethereum_address, key_entry->info.ethereum_address, SR_ETHEREUM_ADDRESS_SIZE);
    
    g_operation_count++;
    
    DMSG("Returned public key for: %s", key_entry->key_id);
    
    return TEE_SUCCESS;
}

static TEE_Result cmd_list_keys(uint32_t param_types, TEE_Param params[4])
{
    sr_key_list_result_t *result;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        EMSG("Invalid parameter types for list_keys");
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    result = (sr_key_list_result_t *)params[0].memref.buffer;
    
    if (params[0].memref.size < sizeof(sr_key_list_result_t)) {
        EMSG("Output buffer too small for key list result");
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    /* Return key list */
    memset(result, 0, sizeof(sr_key_list_result_t));
    result->key_count = g_key_count;
    memcpy(result->keys, g_keys, g_key_count * sizeof(sr_key_info_t));
    
    g_operation_count++;
    
    DMSG("Listed %u keys", g_key_count);
    
    return TEE_SUCCESS;
}

static TEE_Result cmd_get_version(uint32_t param_types, TEE_Param params[4])
{
    sr_version_info_t *result;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    result = (sr_version_info_t *)params[0].memref.buffer;
    
    if (params[0].memref.size < sizeof(sr_version_info_t)) {
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    /* Return version information */
    memset(result, 0, sizeof(sr_version_info_t));
    result->major = 1;
    result->minor = 0;
    result->patch = 0;
    strncpy(result->build_info, "SuperRelay TA v1.0.0", sizeof(result->build_info) - 1);
    
    return TEE_SUCCESS;
}

static TEE_Result cmd_health_check(uint32_t param_types, TEE_Param params[4])
{
    sr_health_result_t *result;
    
    /* Check parameter types */
    uint32_t exp_param_types = TEE_PARAM_TYPES(TEE_PARAM_TYPE_MEMREF_OUTPUT,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE,
                                               TEE_PARAM_TYPE_NONE);
    
    if (param_types != exp_param_types) {
        return TEE_ERROR_BAD_PARAMETERS;
    }
    
    result = (sr_health_result_t *)params[0].memref.buffer;
    
    if (params[0].memref.size < sizeof(sr_health_result_t)) {
        return TEE_ERROR_SHORT_BUFFER;
    }
    
    /* Return health information */
    memset(result, 0, sizeof(sr_health_result_t));
    result->status = SR_SUCCESS;
    result->active_sessions = g_session_count;
    result->total_operations = g_operation_count;
    result->storage_usage = g_key_count * sizeof(sr_key_entry_t);
    result->uptime = get_current_time() - g_start_time;
    
    return TEE_SUCCESS;
}

/*
 * TA Entry Points
 */

TEE_Result TA_CreateEntryPoint(void)
{
    IMSG("SuperRelay TA Create Entry Point");
    
    /* Initialize global state */
    memset(g_keys, 0, sizeof(g_keys));
    memset(g_key_storage, 0, sizeof(g_key_storage));
    g_key_count = 0;
    g_session_count = 0;
    g_operation_count = 0;
    g_start_time = get_current_time();
    
    return TEE_SUCCESS;
}

void TA_DestroyEntryPoint(void)
{
    IMSG("SuperRelay TA Destroy Entry Point");
    
    /* Clear sensitive data */
    memset(g_key_storage, 0, sizeof(g_key_storage));
    memset(g_keys, 0, sizeof(g_keys));
}

TEE_Result TA_OpenSessionEntryPoint(uint32_t param_types,
                                    TEE_Param __maybe_unused params[4],
                                    void __maybe_unused **sess_ctx)
{
    (void)&param_types;
    (void)&params;
    (void)&sess_ctx;
    
    g_session_count++;
    
    IMSG("SuperRelay TA Open Session (total sessions: %u)", g_session_count);
    
    return TEE_SUCCESS;
}

void TA_CloseSessionEntryPoint(void __maybe_unused *sess_ctx)
{
    (void)&sess_ctx;
    
    if (g_session_count > 0)
        g_session_count--;
    
    IMSG("SuperRelay TA Close Session (remaining sessions: %u)", g_session_count);
}

TEE_Result TA_InvokeCommandEntryPoint(void __maybe_unused *sess_ctx,
                                      uint32_t cmd_id,
                                      uint32_t param_types,
                                      TEE_Param params[4])
{
    (void)&sess_ctx;
    
    DMSG("SuperRelay TA Invoke Command: %u", cmd_id);
    
    switch (cmd_id) {
    case TA_SUPER_RELAY_CMD_GENERATE_KEY:
        return cmd_generate_key(param_types, params);
        
    case TA_SUPER_RELAY_CMD_SIGN_MESSAGE:
        return cmd_sign_message(param_types, params);
        
    case TA_SUPER_RELAY_CMD_GET_PUBLIC_KEY:
        return cmd_get_public_key(param_types, params);
        
    case TA_SUPER_RELAY_CMD_LIST_KEYS:
        return cmd_list_keys(param_types, params);
        
    case TA_SUPER_RELAY_CMD_GET_VERSION:
        return cmd_get_version(param_types, params);
        
    case TA_SUPER_RELAY_CMD_HEALTH_CHECK:
        return cmd_health_check(param_types, params);
        
    case TA_SUPER_RELAY_CMD_IMPORT_KEY:
    case TA_SUPER_RELAY_CMD_DELETE_KEY:
        /* TODO: Implement these commands */
        EMSG("Command not yet implemented: %u", cmd_id);
        return TEE_ERROR_NOT_IMPLEMENTED;
        
    default:
        EMSG("Unknown command: %u", cmd_id);
        return TEE_ERROR_BAD_PARAMETERS;
    }
}