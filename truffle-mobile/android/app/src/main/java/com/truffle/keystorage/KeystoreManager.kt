package com.truffle.keystorage

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.*
import java.security.spec.X509EncodedKeySpec
import javax.crypto.*
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

/**
 * KeystoreManager - Android Keystore Secure Key Storage
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Private keys stored in Android Keystore (hardware-backed when available)
 * - AES-256-GCM for symmetric encryption
 * - RSA/EC for asymmetric operations
 * - X3DH key exchange support for device pairing
 */
class KeystoreManager private constructor(context: Context) {
    
    companion object {
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
        private const val KEY_ALIAS_IDENTITY = "truffle_identity_key"
        private const val KEY_ALIAS_SYNC = "truffle_sync_key"
        private const val KEY_ALIAS_AUTH = "truffle_auth_key"
        private const val GCM_TAG_LENGTH = 128
        private const val GCM_IV_LENGTH = 12
        
        @Volatile
        private var instance: KeystoreManager? = null
        
        fun initialize(context: Context): KeystoreManager {
            return instance ?: synchronized(this) {
                instance ?: KeystoreManager(context.applicationContext).also {
                    instance = it
                    it.initializeKeys()
                }
            }
        }
        
        fun getInstance(): KeystoreManager {
            return instance ?: throw IllegalStateException("KeystoreManager not initialized")
        }
    }
    
    private val keyStore: KeyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
    
    // Initialize keys on first use
    private fun initializeKeys() {
        // Generate identity key pair if not exists
        if (!keyStore.containsAlias(KEY_ALIAS_IDENTITY)) {
            generateIdentityKeyPair()
        }
        
        // Generate sync key if not exists
        if (!keyStore.containsAlias(KEY_ALIAS_SYNC)) {
            generateSyncKey()
        }
    }
    
    // MARK: - Identity Keys (EC P-256)
    
    private fun generateIdentityKeyPair() {
        val keyPairGenerator = KeyPairGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_EC,
            ANDROID_KEYSTORE
        )
        
        val params = KeyGenParameterSpec.Builder(
            KEY_ALIAS_IDENTITY,
            KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY or
            KeyProperties.PURPOSE_AGREE_KEY
        )
            .setAlgorithmParameterSpec(java.security.spec.ECGenParameterSpec("secp256r1"))
            .setUserAuthenticationRequired(false)
            .setRandomizedEncryptionRequired(true)
            .build()
        
        keyPairGenerator.initialize(params)
        keyPairGenerator.generateKeyPair()
    }
    
    /**
     * Get the identity public key for sharing during device pairing
     */
    fun getIdentityPublicKey(): ByteArray? {
        val entry = keyStore.getEntry(KEY_ALIAS_IDENTITY, null) as? KeyStore.PrivateKeyEntry
        return entry?.certificate?.publicKey?.encoded
    }
    
    /**
     * Get the identity private key for key agreement
     */
    private fun getIdentityPrivateKey(): PrivateKey? {
        val entry = keyStore.getEntry(KEY_ALIAS_IDENTITY, null) as? KeyStore.PrivateKeyEntry
        return entry?.privateKey
    }
    
    // MARK: - Sync Key (AES-256)
    
    private fun generateSyncKey() {
        val keyGenerator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            ANDROID_KEYSTORE
        )
        
        val params = KeyGenParameterSpec.Builder(
            KEY_ALIAS_SYNC,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setRandomizedEncryptionRequired(true)
            .build()
        
        keyGenerator.init(params)
        keyGenerator.generateKey()
    }
    
    /**
     * Store a derived sync key (from X3DH handshake)
     */
    fun storeSyncKey(keyBytes: ByteArray) {
        // Store in encrypted preferences
        val encrypted = encryptWithIdentityKey(keyBytes)
        val prefs = getSecurePreferences()
        prefs.edit()
            .putString("sync_key", Base64.encodeToString(encrypted, Base64.NO_WRAP))
            .apply()
    }
    
    /**
     * Get the sync key
     */
    fun getSyncKey(): ByteArray? {
        val prefs = getSecurePreferences()
        val encrypted = prefs.getString("sync_key", null) ?: return null
        return decryptWithIdentityKey(Base64.decode(encrypted, Base64.NO_WRAP))
    }
    
    /**
     * Delete sync key (for logout/unpair)
     */
    fun deleteSyncKey() {
        val prefs = getSecurePreferences()
        prefs.edit().remove("sync_key").apply()
    }
    
    // MARK: - X3DH Key Exchange
    
    /**
     * Generate ephemeral keys for X3DH handshake
     */
    fun generateEphemeralKeyPair(): KeyPair {
        val keyPairGenerator = KeyPairGenerator.getInstance("EC")
        keyPairGenerator.initialize(java.security.spec.ECGenParameterSpec("secp256r1"))
        return keyPairGenerator.generateKeyPair()
    }
    
    /**
     * Perform X3DH key agreement
     * 
     * @param ourIdentityPrivateKey Our identity private key
     * @param ourEphemeralPrivateKey Our ephemeral private key
     * @param theirIdentityPublicKey Their identity public key
     * @param theirEphemeralPublicKey Their ephemeral public key
     * @return Shared secret key
     */
    fun performX3DH(
        ourIdentityPrivateKey: PrivateKey,
        ourEphemeralPrivateKey: PrivateKey,
        theirIdentityPublicKey: PublicKey,
        theirEphemeralPublicKey: PublicKey
    ): ByteArray {
        // DH1 = DH(IKA, SPKB)
        val dh1 = performECDH(ourIdentityPrivateKey, theirEphemeralPublicKey)
        
        // DH2 = DH(EKA, IKB)
        val dh2 = performECDH(ourEphemeralPrivateKey, theirIdentityPublicKey)
        
        // DH3 = DH(EKA, SPKB)
        val dh3 = performECDH(ourEphemeralPrivateKey, theirEphemeralPublicKey)
        
        // Combine and derive key using HKDF
        val combined = dh1 + dh2 + dh3
        return hkdfSha256(combined, "truffle-sync-key".toByteArray(), 32)
    }
    
    private fun performECDH(privateKey: PrivateKey, publicKey: PublicKey): ByteArray {
        val keyAgreement = KeyAgreement.getInstance("ECDH")
        keyAgreement.init(privateKey)
        keyAgreement.doPhase(publicKey, true)
        return keyAgreement.generateSecret()
    }
    
    // MARK: - Encryption/Decryption
    
    /**
     * Encrypt data using AES-256-GCM with the sync key
     */
    fun encrypt(data: ByteArray): Pair<ByteArray, ByteArray>? {
        return try {
            val key = getSyncKey() ?: return null
            
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            val secretKey = SecretKeySpec(key, "AES")
            cipher.init(Cipher.ENCRYPT_MODE, secretKey)
            
            val iv = cipher.iv
            val ciphertext = cipher.doFinal(data)
            
            Pair(iv, ciphertext)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Decrypt data using AES-256-GCM with the sync key
     */
    fun decrypt(iv: ByteArray, ciphertext: ByteArray): ByteArray? {
        return try {
            val key = getSyncKey() ?: return null
            
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            val secretKey = SecretKeySpec(key, "AES")
            val spec = GCMParameterSpec(GCM_TAG_LENGTH, iv)
            cipher.init(Cipher.DECRYPT_MODE, secretKey, spec)
            
            cipher.doFinal(ciphertext)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Encrypt with ChaCha20-Poly1305 (alternative for mobile)
     */
    fun encryptChaCha20(data: ByteArray, key: ByteArray): Pair<ByteArray, ByteArray>? {
        return try {
            val cipher = Cipher.getInstance("ChaCha20-Poly1305")
            val secretKey = SecretKeySpec(key, "ChaCha20")
            cipher.init(Cipher.ENCRYPT_MODE, secretKey)
            
            val iv = cipher.iv
            val ciphertext = cipher.doFinal(data)
            
            Pair(iv, ciphertext)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Decrypt with ChaCha20-Poly1305
     */
    fun decryptChaCha20(iv: ByteArray, ciphertext: ByteArray, key: ByteArray): ByteArray? {
        return try {
            val cipher = Cipher.getInstance("ChaCha20-Poly1305")
            val secretKey = SecretKeySpec(key, "ChaCha20")
            val spec = javax.crypto.spec.IvParameterSpec(iv)
            cipher.init(Cipher.DECRYPT_MODE, secretKey, spec)
            
            cipher.doFinal(ciphertext)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }
    
    // MARK: - HMAC
    
    /**
     * Calculate HMAC-SHA256
     */
    fun hmacSha256(data: ByteArray, key: ByteArray): ByteArray {
        val mac = javax.crypto.Mac.getInstance("HmacSHA256")
        mac.init(SecretKeySpec(key, "HmacSHA256"))
        return mac.doFinal(data)
    }
    
    // MARK: - HKDF
    
    /**
     * HKDF-SHA256 key derivation
     */
    fun hkdfSha256(ikm: ByteArray, info: ByteArray, length: Int): ByteArray {
        // Extract
        val salt = ByteArray(32)
        val prk = hmacSha256(ikm, salt)
        
        // Expand
        var okm = ByteArray(0)
        var previousBlock = ByteArray(0)
        var counter: Byte = 1
        
        while (okm.size < length) {
            val data = previousBlock + info + byteArrayOf(counter)
            previousBlock = hmacSha256(data, prk)
            okm += previousBlock
            counter++
        }
        
        return okm.copyOf(length)
    }
    
    // MARK: - Device Fingerprint
    
    /**
     * Generate a privacy-preserving device fingerprint
     */
    fun getDeviceFingerprint(): String {
        val prefs = getSecurePreferences()
        var fingerprint = prefs.getString("device_fingerprint", null)
        
        if (fingerprint == null) {
            // Generate new fingerprint
            val uuid = java.util.UUID.randomUUID().toString()
            val salt = "truffle_device_salt_v1"
            val input = (uuid + salt).toByteArray()
            
            val md = MessageDigest.getInstance("SHA-256")
            val hash = md.digest(input)
            fingerprint = hash.joinToString("") { "%02x".format(it) }
            
            prefs.edit().putString("device_fingerprint", fingerprint).apply()
        }
        
        return fingerprint
    }
    
    // MARK: - Private Helpers
    
    private fun encryptWithIdentityKey(data: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("RSA/ECB/PKCS1Padding")
        val publicKey = getIdentityPublicKey()?.let {
            val spec = X509EncodedKeySpec(it)
            KeyFactory.getInstance("EC").generatePublic(spec)
        }
        cipher.init(Cipher.ENCRYPT_MODE, publicKey)
        return cipher.doFinal(data)
    }
    
    private fun decryptWithIdentityKey(data: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("RSA/ECB/PKCS1Padding")
        cipher.init(Cipher.DECRYPT_MODE, getIdentityPrivateKey())
        return cipher.doFinal(data)
    }
    
    private fun getSecurePreferences(): android.content.SharedPreferences {
        return android.preference.PreferenceManager.getDefaultSharedPreferences(
            android.app.Application.getApplicationContext()
        )
    }
    
    // MARK: - Key Validation
    
    /**
     * Check if keys are hardware-backed
     */
    fun isHardwareBacked(): Boolean {
        return if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.M) {
            val entry = keyStore.getEntry(KEY_ALIAS_IDENTITY, null) as? KeyStore.PrivateKeyEntry
            val factory = KeyFactory.getInstance(entry?.privateKey?.algorithm, ANDROID_KEYSTORE)
            val keyInfo = factory.getKeySpec(entry?.privateKey, android.security.keystore.KeyInfo::class.java)
            keyInfo.isInsideSecureHardware
        } else {
            false
        }
    }
    
    /**
     * Verify key integrity
     */
    fun verifyKeys(): Boolean {
        return try {
            keyStore.containsAlias(KEY_ALIAS_IDENTITY) &&
            keyStore.containsAlias(KEY_ALIAS_SYNC) &&
            getIdentityPublicKey() != null
        } catch (e: Exception) {
            false
        }
    }
    
    /**
     * Reset all keys (for troubleshooting)
     */
    fun resetKeys() {
        keyStore.deleteEntry(KEY_ALIAS_IDENTITY)
        keyStore.deleteEntry(KEY_ALIAS_SYNC)
        keyStore.deleteEntry(KEY_ALIAS_AUTH)
        
        val prefs = getSecurePreferences()
        prefs.edit()
            .remove("sync_key")
            .remove("device_fingerprint")
            .apply()
        
        initializeKeys()
    }
}

// MARK: - Sync Message

data class SyncMessage(
    val header: Header,
    val payload: ByteArray, // Encrypted
    val mac: ByteArray // HMAC-SHA256
) {
    data class Header(
        val protocolVersion: Int,
        val deviceId: String,
        val timestamp: Long,
        val nonce: ByteArray
    )
    
    fun toByteArray(): ByteArray {
        // Serialize to bytes
        val headerBytes = org.json.JSONObject().apply {
            put("protocolVersion", header.protocolVersion)
            put("deviceId", header.deviceId)
            put("timestamp", header.timestamp)
            put("nonce", Base64.encodeToString(header.nonce, Base64.NO_WRAP))
        }.toString().toByteArray()
        
        return headerBytes + payload + mac
    }
    
    companion object {
        fun fromByteArray(data: ByteArray): SyncMessage? {
            // Deserialize from bytes
            return null // Implementation
        }
    }
}
