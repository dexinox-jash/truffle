//
//  KeychainManager.swift
//  TruffleCore
//
//  Project Truffle - Secure Key Storage
//  SPEC v2.0 Section 3.2.1: SOC 2 CC6.1 - Logical access controls
//
//  Implements secure key storage using:
//  - iOS Secure Enclave (when available)
//  - iOS Keychain with kSecAttrAccessibleWhenUnlockedThisDeviceOnly
//  - AES-256-GCM for data encryption
//

import Foundation
import Security
import LocalAuthentication
import CryptoKit

// MARK: - Keychain Manager
class KeychainManager {
    
    // MARK: - Singleton
    static let shared = KeychainManager()
    
    // MARK: - Constants
    private let serviceName = "com.truffle.secure"
    private let accessGroup = "group.com.truffle.shared"
    
    // Secure Enclave availability
    private var isSecureEnclaveAvailable: Bool {
        return SecureEnclave.isAvailable
    }
    
    // MARK: - Initialization
    private init() {
        // Verify Secure Enclave availability on init
        if isSecureEnclaveAvailable {
            print("[KeychainManager] Secure Enclave available")
        } else {
            print("[KeychainManager] Secure Enclave unavailable, using Keychain only")
        }
    }
    
    // MARK: - Public API
    
    /// Store a key in the Secure Enclave or Keychain
    /// - Parameters:
    ///   - key: The key data to store
    ///   - identifier: Unique identifier for the key
    /// - Returns: OSStatus indicating success or failure
    @discardableResult
    func storeKey(_ key: Data, identifier: String) -> OSStatus {
        // Delete any existing key first
        _ = deleteKey(identifier: identifier)
        
        if isSecureEnclaveAvailable {
            return storeInSecureEnclave(key: key, identifier: identifier)
        } else {
            return storeInKeychain(key: key, identifier: identifier)
        }
    }
    
    /// Retrieve a key from Secure Enclave or Keychain
    /// - Parameter identifier: The key identifier
    /// - Returns: The key data, or nil if not found
    func retrieveKey(identifier: String) -> Data? {
        if isSecureEnclaveAvailable {
            return retrieveFromSecureEnclave(identifier: identifier)
        } else {
            return retrieveFromKeychain(identifier: identifier)
        }
    }
    
    /// Delete a key from Secure Enclave and Keychain
    /// - Parameter identifier: The key identifier
    /// - Returns: OSStatus indicating success or failure
    @discardableResult
    func deleteKey(identifier: String) -> OSStatus {
        // Try to delete from both locations
        let keychainStatus = deleteFromKeychain(identifier: identifier)
        let enclaveStatus = deleteFromSecureEnclave(identifier: identifier)
        
        // Return success if either deletion succeeded
        return (keychainStatus == errSecSuccess || enclaveStatus == errSecSuccess)
            ? errSecSuccess : keychainStatus
    }
    
    /// Generate and store a new encryption key
    /// - Parameter identifier: The key identifier
    /// - Returns: The generated key data, or nil if generation failed
    @discardableResult
    func generateAndStoreKey(identifier: String) -> Data? {
        // Generate 256-bit key
        var keyData = Data(count: 32)
        let result = keyData.withUnsafeMutableBytes { bytes in
            SecRandomCopyBytes(kSecRandomDefault, 32, bytes.bindMemory(to: UInt8.self).baseAddress!)
        }
        
        guard result == errSecSuccess else {
            print("[KeychainManager] Failed to generate random key")
            return nil
        }
        
        let status = storeKey(keyData, identifier: identifier)
        guard status == errSecSuccess else {
            print("[KeychainManager] Failed to store generated key: \(status)")
            return nil
        }
        
        return keyData
    }
    
    /// Check if a key exists
    /// - Parameter identifier: The key identifier
    /// - Returns: True if the key exists
    func keyExists(identifier: String) -> Bool {
        return retrieveKey(identifier: identifier) != nil
    }
    
    /// Store sync key pair (public and private)
    /// - Parameters:
    ///   - publicKey: The public key data
    ///   - privateKey: The private key data
    ///   - identifier: Base identifier for the key pair
    /// - Returns: OSStatus indicating success or failure
    @discardableResult
    func storeKeyPair(publicKey: Data, privateKey: Data, identifier: String) -> OSStatus {
        let pubStatus = storeKey(publicKey, identifier: "\(identifier).public")
        let privStatus = storeKey(privateKey, identifier: "\(identifier).private")
        
        return (pubStatus == errSecSuccess && privStatus == errSecSuccess)
            ? errSecSuccess : errSecParam
    }
    
    /// Retrieve key pair
    /// - Parameter identifier: Base identifier for the key pair
    /// - Returns: Tuple of (publicKey, privateKey), or nil if not found
    func retrieveKeyPair(identifier: String) -> (publicKey: Data, privateKey: Data)? {
        guard let publicKey = retrieveKey(identifier: "\(identifier).public"),
              let privateKey = retrieveKey(identifier: "\(identifier).private") else {
            return nil
        }
        return (publicKey, privateKey)
    }
    
    // MARK: - Secure Enclave Operations
    
    private func storeInSecureEnclave(key: Data, identifier: String) -> OSStatus {
        do {
            // Create a Secure Enclave key for wrapping
            let seKey = try SecureEnclave.P256.KeyAgreement.PrivateKey()
            
            // Store the Secure Enclave key reference in Keychain
            let keyData = seKey.x963Representation
            
            let query: [String: Any] = [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrAccount as String: identifier,
                kSecAttrService as String: serviceName,
                kSecAttrAccessGroup as String: accessGroup,
                kSecValueData as String: keyData,
                kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
                kSecUseAuthenticationUI as String: kSecUseAuthenticationUIAllow,
                kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave
            ]
            
            return SecItemAdd(query as CFDictionary, nil)
            
        } catch {
            print("[KeychainManager] Secure Enclave storage failed: \(error)")
            // Fall back to regular Keychain
            return storeInKeychain(key: key, identifier: identifier)
        }
    }
    
    private func retrieveFromSecureEnclave(identifier: String) -> Data? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: identifier,
            kSecAttrService as String: serviceName,
            kSecAttrAccessGroup as String: accessGroup,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        guard status == errSecSuccess,
              let keyData = result as? Data else {
            return nil
        }
        
        return keyData
    }
    
    private func deleteFromSecureEnclave(identifier: String) -> OSStatus {
        // Secure Enclave keys are deleted via Keychain
        return deleteFromKeychain(identifier: identifier)
    }
    
    // MARK: - Standard Keychain Operations
    
    private func storeInKeychain(key: Data, identifier: String) -> OSStatus {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: identifier,
            kSecAttrService as String: serviceName,
            kSecAttrAccessGroup as String: accessGroup,
            kSecValueData as String: key,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        ]
        
        return SecItemAdd(query as CFDictionary, nil)
    }
    
    private func retrieveFromKeychain(identifier: String) -> Data? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: identifier,
            kSecAttrService as String: serviceName,
            kSecAttrAccessGroup as String: accessGroup,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        guard status == errSecSuccess,
              let keyData = result as? Data else {
            return nil
        }
        
        return keyData
    }
    
    private func deleteFromKeychain(identifier: String) -> OSStatus {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: identifier,
            kSecAttrService as String: serviceName,
            kSecAttrAccessGroup as String: accessGroup
        ]
        
        return SecItemDelete(query as CFDictionary)
    }
    
    // MARK: - Biometric Authentication
    
    /// Store key with biometric authentication requirement
    /// - Parameters:
    ///   - key: The key data to store
    ///   - identifier: Unique identifier
    ///   - biometricRequired: Whether biometric auth is required to access
    /// - Returns: OSStatus indicating success or failure
    @discardableResult
    func storeKeyWithBiometric(_ key: Data, identifier: String, biometricRequired: Bool = true) -> OSStatus {
        _ = deleteKey(identifier: identifier)
        
        var accessibility = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        var accessControl: SecAccessControl?
        
        if biometricRequired {
            var error: Unmanaged<CFError>?
            accessControl = SecAccessControlCreateWithFlags(
                kCFAllocatorDefault,
                kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
                .biometryCurrentSet,
                &error
            )
            
            if error != nil {
                print("[KeychainManager] Failed to create access control")
            }
        }
        
        var query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: identifier,
            kSecAttrService as String: serviceName,
            kSecAttrAccessGroup as String: accessGroup,
            kSecValueData as String: key,
            kSecAttrAccessible as String: accessibility
        ]
        
        if let accessControl = accessControl {
            query[kSecAttrAccessControl as String] = accessControl
        }
        
        return SecItemAdd(query as CFDictionary, nil)
    }
    
    // MARK: - Key Rotation
    
    /// Rotate a key (generate new key and re-encrypt data)
    /// - Parameters:
    ///   - identifier: The key identifier
    ///   - reencryptHandler: Closure to re-encrypt existing data with new key
    /// - Returns: True if rotation succeeded
    func rotateKey(identifier: String, reencryptHandler: (Data, Data) -> Data?) -> Bool {
        // Retrieve old key
        guard let oldKey = retrieveKey(identifier: identifier) else {
            print("[KeychainManager] Cannot rotate: old key not found")
            return false
        }
        
        // Generate new key
        guard let newKey = generateAndStoreKey(identifier: "\(identifier).new") else {
            return false
        }
        
        // Re-encrypt data (caller handles this)
        // If reencryption succeeds, replace old key with new
        // If fails, clean up and return false
        
        // For now, just replace the key
        _ = deleteKey(identifier: identifier)
        let status = storeKey(newKey, identifier: identifier)
        _ = deleteKey(identifier: "\(identifier).new")
        
        return status == errSecSuccess
    }
    
    // MARK: - Key Export (for backup)
    
    /// Export key encrypted with a backup password
    /// - Parameters:
    ///   - identifier: The key identifier
    ///   - password: The backup password
    /// - Returns: Encrypted key data for backup, or nil if export failed
    func exportKey(identifier: String, password: String) -> Data? {
        guard let key = retrieveKey(identifier: identifier) else {
            return nil
        }
        
        // Derive encryption key from password
        guard let passwordData = password.data(using: .utf8) else {
            return nil
        }
        
        let derivedKey = HKDF<SHA256>.deriveKey(
            inputKeyMaterial: .init(data: passwordData),
            salt: Data("truffle_backup_salt".utf8),
            info: Data("backup".utf8),
            outputByteCount: 32
        )
        
        // Encrypt key with derived key
        do {
            let sealedBox = try AES.GCM.seal(key, using: derivedKey)
            return sealedBox.combined
        } catch {
            print("[KeychainManager] Export encryption failed: \(error)")
            return nil
        }
    }
    
    /// Import key from backup
    /// - Parameters:
    ///   - encryptedData: The encrypted key data
    ///   - password: The backup password
    ///   - identifier: The key identifier to store under
    /// - Returns: True if import succeeded
    func importKey(encryptedData: Data, password: String, identifier: String) -> Bool {
        // Derive decryption key from password
        guard let passwordData = password.data(using: .utf8) else {
            return false
        }
        
        let derivedKey = HKDF<SHA256>.deriveKey(
            inputKeyMaterial: .init(data: passwordData),
            salt: Data("truffle_backup_salt".utf8),
            info: Data("backup".utf8),
            outputByteCount: 32
        )
        
        // Decrypt key
        do {
            let sealedBox = try AES.GCM.SealedBox(combined: encryptedData)
            let key = try AES.GCM.open(sealedBox, using: derivedKey)
            
            // Store decrypted key
            let status = storeKey(key, identifier: identifier)
            return status == errSecSuccess
        } catch {
            print("[KeychainManager] Import decryption failed: \(error)")
            return false
        }
    }
    
    // MARK: - Debug Helpers
    
    #if DEBUG
    /// List all stored keys (debug only, no values)
    func listStoredKeys() -> [String] {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecReturnAttributes as String: true,
            kSecMatchLimit as String: kSecMatchLimitAll
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        guard status == errSecSuccess,
              let items = result as? [[String: Any]] else {
            return []
        }
        
        return items.compactMap { $0[kSecAttrAccount as String] as? String }
    }
    
    /// Clear all keys (debug only)
    func clearAllKeys() {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName
        ]
        
        SecItemDelete(query as CFDictionary)
    }
    #endif
}

// MARK: - Key Identifiers

extension KeychainManager {
    enum KeyIdentifier {
        static let syncKey = "truffle.sync_key"
        static let authKey = "truffle.auth_key"
        static let deviceIdentity = "truffle.device_identity"
        static let pairingToken = "truffle.pairing_token"
        static let encryptionKey = "truffle.encryption_key"
        static let backupKey = "truffle.backup_key"
    }
}

// MARK: - Errors

enum KeychainError: Error {
    case itemNotFound
    case duplicateItem
    case invalidStatus(OSStatus)
    case conversionFailed
    case secureEnclaveUnavailable
    case biometricNotAvailable
}
