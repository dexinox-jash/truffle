//
//  RawStorageManager.swift
//  TruffleCore
//
//  Project Truffle - Raw Storage Manager
//  SPEC v2.0 Section 2.1.1: Raw Entity Storage
//
//  Manages raw screenshot storage:
//  - File system operations
//  - Storage quota enforcement
//  - Thumbnail generation
//

import Foundation
import UIKit

// MARK: - Raw Storage Manager
class RawStorageManager {
    
    // MARK: - Singleton
    static let shared = RawStorageManager()
    
    // MARK: - Constants
    private let rawStoragePath = "raw"
    private let thumbnailPath = "thumbnails"
    private let maxStorageBytes: UInt64 = 10 * 1024 * 1024 * 1024 // 10GB
    private let warningThreshold: UInt64 = 5 * 1024 * 1024 * 1024 // 5GB
    private let thumbnailSize = CGSize(width: 200, height: 200)
    
    // MARK: - Properties
    private var storageURL: URL {
        let documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        return documents.appendingPathComponent(rawStoragePath)
    }
    
    private var thumbnailURL: URL {
        let documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        return documents.appendingPathComponent(thumbnailPath)
    }
    
    // MARK: - Initialization
    private init() {
        createStorageDirectories()
    }
    
    // MARK: - Public API
    
    /// Save artifact to raw storage
    func saveArtifact(_ artifact: RawArtifact) throws {
        // Check storage limits
        let currentUsage = getStorageUsage()
        guard currentUsage + artifact.metadata.sizeBytes <= maxStorageBytes else {
            throw StorageError.quotaExceeded
        }
        
        // Create directory for artifact
        let artifactDir = storageURL.appendingPathComponent(artifact.uuid.uuidString)
        try FileManager.default.createDirectory(at: artifactDir, withIntermediateDirectories: true)
        
        // Save image data
        let imagePath = artifactDir.appendingPathComponent(artifact.filename)
        try artifact.binary.write(to: imagePath)
        
        // Save metadata
        let metadataPath = artifactDir.appendingPathComponent("metadata.json")
        let metadataData = try JSONEncoder().encode(artifact.metadata)
        try metadataData.write(to: metadataPath)
        
        // Generate and save thumbnail
        if let image = UIImage(data: artifact.binary),
           let thumbnail = generateThumbnail(from: image) {
            let thumbnailPath = thumbnailURL.appendingPathComponent("\(artifact.uuid.uuidString).jpg")
            try thumbnail.jpegData(compressionQuality: 0.8)?.write(to: thumbnailPath)
        }
        
        // Post notification
        NotificationCenter.default.post(
            name: .artifactSaved,
            object: nil,
            userInfo: ["uuid": artifact.uuid]
        )
        
        // Check if warning threshold reached
        let newUsage = currentUsage + artifact.metadata.sizeBytes
        if newUsage >= warningThreshold && currentUsage < warningThreshold {
            NotificationCenter.default.post(
                name: .storageWarning,
                object: nil,
                userInfo: ["usage": newUsage]
            )
        }
    }
    
    /// Load image from raw storage
    func loadImage(uuid: UUID) -> UIImage? {
        let imagePath = storageURL
            .appendingPathComponent(uuid.uuidString)
            .appendingPathComponent("image.jpg")
        
        guard FileManager.default.fileExists(atPath: imagePath.path) else {
            return nil
        }
        
        return UIImage(contentsOfFile: imagePath.path)
    }
    
    /// Load thumbnail for artifact
    func loadThumbnail(uuid: UUID) -> UIImage? {
        let thumbnailPath = thumbnailURL.appendingPathComponent("\(uuid.uuidString).jpg")
        
        guard FileManager.default.fileExists(atPath: thumbnailPath.path) else {
            // Try to generate from full image
            if let image = loadImage(uuid: uuid) {
                return generateThumbnail(from: image)
            }
            return nil
        }
        
        return UIImage(contentsOfFile: thumbnailPath.path)
    }
    
    /// Get artifact metadata
    func getMetadata(uuid: UUID) -> RawMetadata? {
        let metadataPath = storageURL
            .appendingPathComponent(uuid.uuidString)
            .appendingPathComponent("metadata.json")
        
        guard FileManager.default.fileExists(atPath: metadataPath.path),
              let data = try? Data(contentsOf: metadataPath) else {
            return nil
        }
        
        return try? JSONDecoder().decode(RawMetadata.self, from: data)
    }
    
    /// Delete artifact
    func deleteArtifact(uuid: UUID) throws {
        let artifactDir = storageURL.appendingPathComponent(uuid.uuidString)
        let thumbnailPath = thumbnailURL.appendingPathComponent("\(uuid.uuidString).jpg")
        
        // Delete artifact directory
        if FileManager.default.fileExists(atPath: artifactDir.path) {
            try FileManager.default.removeItem(at: artifactDir)
        }
        
        // Delete thumbnail
        if FileManager.default.fileExists(atPath: thumbnailPath.path) {
            try FileManager.default.removeItem(at: thumbnailPath)
        }
        
        NotificationCenter.default.post(
            name: .artifactDeleted,
            object: nil,
            userInfo: ["uuid": uuid]
        )
    }
    
    /// Get total storage usage
    func getStorageUsage() -> UInt64 {
        return calculateDirectorySize(storageURL)
    }
    
    /// Get storage breakdown
    func getStorageBreakdown() -> StorageBreakdown {
        let rawSize = calculateDirectorySize(storageURL)
        let thumbnailSize = calculateDirectorySize(thumbnailURL)
        
        return StorageBreakdown(
            rawStorage: rawSize,
            thumbnails: thumbnailSize,
            total: rawSize + thumbnailSize
        )
    }
    
    /// List all artifacts
    func listArtifacts() -> [RawArtifactListing] {
        guard let contents = try? FileManager.default.contentsOfDirectory(
            at: storageURL,
            includingPropertiesForKeys: [.creationDateKey],
            options: .skipsHiddenFiles
        ) else {
            return []
        }
        
        return contents.compactMap { url in
            let uuid = url.lastPathComponent
            guard let uuidObj = UUID(uuidString: uuid) else { return nil }
            
            guard let metadata = getMetadata(uuid: uuidObj) else { return nil }
            
            let attributes = try? FileManager.default.attributesOfItem(atPath: url.path)
            let creationDate = attributes?[.creationDate] as? Date
            
            return RawArtifactListing(
                uuid: uuidObj,
                filename: url.appendingPathComponent("image.jpg").lastPathComponent,
                createdAt: creationDate ?? Date(),
                sizeBytes: metadata.sizeBytes,
                thumbnail: loadThumbnail(uuid: uuidObj)
            )
        }.sorted { $0.createdAt > $1.createdAt }
    }
    
    /// Clean up old artifacts (keep only last N)
    func cleanupOldArtifacts(keepCount: Int) throws {
        let artifacts = listArtifacts()
        
        guard artifacts.count > keepCount else { return }
        
        let toDelete = artifacts.suffix(from: keepCount)
        for artifact in toDelete {
            try deleteArtifact(uuid: artifact.uuid)
        }
    }
    
    /// Export artifact to external location
    func exportArtifact(uuid: UUID, to destination: URL) throws {
        let sourcePath = storageURL
            .appendingPathComponent(uuid.uuidString)
            .appendingPathComponent("image.jpg")
        
        guard FileManager.default.fileExists(atPath: sourcePath.path) else {
            throw StorageError.artifactNotFound
        }
        
        try FileManager.default.copyItem(at: sourcePath, to: destination)
    }
    
    // MARK: - Private Methods
    
    private func createStorageDirectories() {
        try? FileManager.default.createDirectory(at: storageURL, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: thumbnailURL, withIntermediateDirectories: true)
    }
    
    private func generateThumbnail(from image: UIImage) -> UIImage? {
        UIGraphicsBeginImageContextWithOptions(thumbnailSize, false, 0.0)
        defer { UIGraphicsEndImageContext() }
        
        // Calculate aspect fill rect
        let imageSize = image.size
        let scale = max(thumbnailSize.width / imageSize.width, thumbnailSize.height / imageSize.height)
        let scaledSize = CGSize(width: imageSize.width * scale, height: imageSize.height * scale)
        let origin = CGPoint(
            x: (thumbnailSize.width - scaledSize.width) / 2,
            y: (thumbnailSize.height - scaledSize.height) / 2
        )
        
        image.draw(in: CGRect(origin: origin, size: scaledSize))
        return UIGraphicsGetImageFromCurrentImageContext()
    }
    
    private func calculateDirectorySize(_ url: URL) -> UInt64 {
        guard let enumerator = FileManager.default.enumerator(
            at: url,
            includingPropertiesForKeys: [.fileSizeKey],
            options: .skipsHiddenFiles
        ) else {
            return 0
        }
        
        var totalSize: UInt64 = 0
        
        for case let fileURL as URL in enumerator {
            if let attributes = try? fileURL.resourceValues(forKeys: [.fileSizeKey]),
               let size = attributes.fileSize {
                totalSize += UInt64(size)
            }
        }
        
        return totalSize
    }
}

// MARK: - Supporting Types

struct RawArtifact {
    let uuid: UUID
    let filename: String
    let binary: Data
    let capturedAt: String
    let deviceId: String
    let appContext: AppContext?
    let geohash: String?
    let metadata: RawMetadata
}

struct RawMetadata: Codable {
    let sizeBytes: UInt64
    let width: Int
    let height: Int
    let ocrText: String?
    let embeddingVector: [Float]?
}

struct AppContext: Codable {
    let bundleId: String
    let appName: String
    let windowTitle: String?
}

struct RawArtifactListing: Identifiable {
    let id = UUID()
    let uuid: UUID
    let filename: String
    let createdAt: Date
    let sizeBytes: UInt64
    let thumbnail: UIImage?
    
    var sizeString: String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(sizeBytes))
    }
}

struct StorageBreakdown {
    let rawStorage: UInt64
    let thumbnails: UInt64
    let total: UInt64
}

enum StorageError: Error {
    case quotaExceeded
    case artifactNotFound
    case invalidImage
    case writeFailed
    
    var localizedDescription: String {
        switch self {
        case .quotaExceeded:
            return "Storage quota exceeded (10GB). Please export or delete screenshots."
        case .artifactNotFound:
            return "Screenshot not found."
        case .invalidImage:
            return "Invalid image data."
        case .writeFailed:
            return "Failed to save screenshot."
        }
    }
}

// MARK: - Notification Names

extension Notification.Name {
    static let artifactSaved = Notification.Name("artifactSaved")
    static let artifactDeleted = Notification.Name("artifactDeleted")
    static let storageWarning = Notification.Name("storageWarning")
}
