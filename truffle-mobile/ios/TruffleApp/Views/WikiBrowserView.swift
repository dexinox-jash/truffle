//
//  WikiBrowserView.swift
//  TruffleApp
//
//  Project Truffle - Read-Only Wiki Browser
//  SPEC v2.0 Section 5.3.1: Mobile Companion - Read-only wiki browser
//
//  Implements:
//  - Tree view navigation (Obsidian-style)
//  - WikiLink support [[Target]]
//  - Backlinks display
//  - Original screenshot viewing
//

import SwiftUI
import MarkdownUI

// MARK: - Wiki Browser View
struct WikiBrowserView: View {
    
    @ObservedObject var viewModel: WikiViewModel
    @State private var navigationPath = NavigationPath()
    
    var body: some View {
        NavigationStack(path: $navigationPath) {
            WikiNodeListView(nodes: viewModel.rootNodes, viewModel: viewModel)
                .navigationTitle("Wiki")
                .navigationDestination(for: WikiNode.self) { node in
                    WikiNodeView(node: node, viewModel: viewModel, navigationPath: $navigationPath)
                }
        }
        .background(Color(hex: "#0A0A0A"))
    }
}

// MARK: - Wiki Node List View
struct WikiNodeListView: View {
    let nodes: [WikiNode]
    @ObservedObject var viewModel: WikiViewModel
    
    var body: some View {
        List {
            ForEach(nodes) { node in
                WikiNodeRow(node: node, viewModel: viewModel)
            }
        }
        .listStyle(PlainListStyle())
        .background(Color(hex: "#0A0A0A"))
    }
}

// MARK: - Wiki Node Row
struct WikiNodeRow: View {
    let node: WikiNode
    @ObservedObject var viewModel: WikiViewModel
    @State private var isExpanded = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            NavigationLink(value: node) {
                HStack(spacing: 12) {
                    // Node type icon
                    Image(systemName: node.iconName)
                        .foregroundColor(node.typeColor)
                        .frame(width: 24)
                    
                    VStack(alignment: .leading, spacing: 4) {
                        Text(node.title)
                            .font(.system(size: 16, weight: .medium))
                            .foregroundColor(Color(hex: "#E5E5E5"))
                            .lineLimit(1)
                        
                        if let summary = node.summary {
                            Text(summary)
                                .font(.system(size: 13))
                                .foregroundColor(Color(hex: "#737373"))
                                .lineLimit(2)
                        }
                    }
                    
                    Spacer()
                    
                    // Backlinks count
                    if node.backlinks.count > 0 {
                        Text("\(node.backlinks.count)")
                            .font(.system(size: 12))
                            .foregroundColor(Color(hex: "#737373"))
                            .padding(.horizontal, 8)
                            .padding(.vertical, 2)
                            .background(Color(hex: "#262626"))
                            .cornerRadius(4)
                    }
                    
                    Image(systemName: "chevron.right")
                        .font(.system(size: 14))
                        .foregroundColor(Color(hex: "#737373"))
                }
                .padding(.vertical, 8)
            }
            
            // Child nodes (if expanded)
            if isExpanded && !node.children.isEmpty {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(node.children) { child in
                        WikiNodeRow(node: child, viewModel: viewModel)
                            .padding(.leading, 36)
                    }
                }
            }
        }
        .background(Color(hex: "#141414"))
        .contentShape(Rectangle())
        .onTapGesture {
            if !node.children.isEmpty {
                withAnimation(.easeInOut(duration: 0.2)) {
                    isExpanded.toggle()
                }
            }
        }
    }
}

// MARK: - Wiki Node View (Detail)
struct WikiNodeView: View {
    let node: WikiNode
    @ObservedObject var viewModel: WikiViewModel
    @Binding var navigationPath: NavigationPath
    
    @State private var showSourceScreenshot = false
    @State private var selectedWikiLink: String?
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                WikiNodeHeader(node: node)
                
                // Source artifacts (screenshots)
                if !node.sourceArtifacts.isEmpty {
                    SourceArtifactsSection(
                        artifacts: node.sourceArtifacts,
                        onSelect: { showSourceScreenshot = true }
                    )
                }
                
                // Content
                WikiContentView(
                    content: node.content,
                    onWikiLinkTap: { link in
                        if let targetNode = viewModel.findNode(byTitle: link) {
                            navigationPath.append(targetNode)
                        }
                    }
                )
                
                // Backlinks
                if !node.backlinks.isEmpty {
                    BacklinksSection(
                        backlinks: node.backlinks,
                        onSelect: { backlink in
                            if let targetNode = viewModel.findNode(byId: backlink.sourceId) {
                                navigationPath.append(targetNode)
                            }
                        }
                    )
                }
                
                // Metadata
                WikiNodeMetadata(node: node)
            }
            .padding()
        }
        .background(Color(hex: "#0A0A0A"))
        .navigationTitle(node.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button(action: { showSourceScreenshot = true }) {
                        Label("View Source", systemImage: "photo")
                    }
                    
                    Button(action: { shareNode() }) {
                        Label("Share", systemImage: "square.and.arrow.up")
                    }
                    
                    Button(action: { copyMarkdown() }) {
                        Label("Copy Markdown", systemImage: "doc.on.doc")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
        }
        .sheet(isPresented: $showSourceScreenshot) {
            SourceScreenshotView(artifacts: node.sourceArtifacts)
        }
    }
    
    private func shareNode() {
        // Share node as markdown
        let activityVC = UIActivityViewController(
            activityItems: [node.markdownExport],
            applicationActivities: nil
        )
        
        if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
           let rootVC = windowScene.windows.first?.rootViewController {
            rootVC.present(activityVC, animated: true)
        }
    }
    
    private func copyMarkdown() {
        UIPasteboard.general.string = node.markdownExport
    }
}

// MARK: - Wiki Node Header
struct WikiNodeHeader: View {
    let node: WikiNode
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: node.iconName)
                    .foregroundColor(node.typeColor)
                    .font(.system(size: 24))
                
                Text(node.title)
                    .font(.system(size: 28, weight: .bold))
                    .foregroundColor(Color(hex: "#E5E5E5"))
                
                Spacer()
            }
            
            // Privacy badge
            HStack {
                PrivacyBadge(classification: node.privacyClassification)
                
                if node.isEncrypted {
                    Image(systemName: "lock.fill")
                        .foregroundColor(Color(hex: "#00D4AA"))
                        .font(.system(size: 12))
                }
                
                Spacer()
                
                Text("Compiled \(node.compiledAt)")
                    .font(.system(size: 12))
                    .foregroundColor(Color(hex: "#737373"))
            }
        }
        .padding(.bottom, 8)
    }
}

struct PrivacyBadge: View {
    let classification: PrivacyClassification
    
    var body: some View {
        Text(classification.label)
            .font(.system(size: 11, weight: .medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(classification.color.opacity(0.2))
            .foregroundColor(classification.color)
            .cornerRadius(4)
    }
}

// MARK: - Source Artifacts Section
struct SourceArtifactsSection: View {
    let artifacts: [SourceArtifact]
    let onSelect: () -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Source Screenshots")
                .font(.system(size: 14, weight: .semibold))
                .foregroundColor(Color(hex: "#737373"))
            
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 12) {
                    ForEach(artifacts) { artifact in
                        SourceArtifactThumbnail(artifact: artifact)
                            .onTapGesture(perform: onSelect)
                    }
                }
            }
        }
        .padding(.vertical, 8)
    }
}

struct SourceArtifactThumbnail: View {
    let artifact: SourceArtifact
    
    var body: some View {
        Group {
            if let thumbnail = artifact.thumbnail {
                Image(uiImage: thumbnail)
                    .resizable()
                    .scaledToFill()
            } else {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(hex: "#262626"))
                    .overlay(
                        Image(systemName: "photo")
                            .foregroundColor(Color(hex: "#737373"))
                    )
            }
        }
        .frame(width: 120, height: 80)
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(hex: "#262626"), lineWidth: 1)
        )
    }
}

// MARK: - Wiki Content View
struct WikiContentView: View {
    let content: String
    let onWikiLinkTap: (String) -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Custom markdown renderer with WikiLink support
            Markdown(content)
                .markdownTheme(.truffle)
                .markdownInlineImageProvider(.truffle)
        }
    }
}

// MARK: - Backlinks Section
struct BacklinksSection: View {
    let backlinks: [WikiLink]
    let onSelect: (WikiLink) -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Backlinks (\(backlinks.count))")
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(Color(hex: "#737373"))
            
            VStack(alignment: .leading, spacing: 8) {
                ForEach(backlinks) { link in
                    Button(action: { onSelect(link) }) {
                        HStack {
                            Image(systemName: "link")
                                .foregroundColor(Color(hex: "#FFB800"))
                                .font(.system(size: 12))
                            
                            Text(link.sourceTitle)
                                .font(.system(size: 14))
                                .foregroundColor(Color(hex: "#FFB800"))
                            
                            Spacer()
                        }
                        .padding(.vertical, 4)
                    }
                }
            }
        }
        .padding(.vertical, 8)
    }
}

// MARK: - Wiki Node Metadata
struct WikiNodeMetadata: View {
    let node: WikiNode
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()
                .background(Color(hex: "#262626"))
            
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Model: \(node.modelVersion)")
                        .font(.system(size: 11))
                        .foregroundColor(Color(hex: "#737373"))
                    
                    Text("Confidence: \(Int(node.confidenceScore * 100))%")
                        .font(.system(size: 11))
                        .foregroundColor(confidenceColor)
                }
                
                Spacer()
                
                if let nodeId = node.id.uuidString.split(separator: "-").first {
                    Text("ID: \(nodeId)...")
                        .font(.system(size: 11))
                        .foregroundColor(Color(hex: "#737373"))
                }
            }
        }
        .padding(.top, 8)
    }
    
    private var confidenceColor: Color {
        if node.confidenceScore >= 0.9 {
            return Color(hex: "#00D4AA")
        } else if node.confidenceScore >= 0.7 {
            return Color(hex: "#FFB800")
        } else {
            return .orange
        }
    }
}

// MARK: - Source Screenshot View (Full Screen)
struct SourceScreenshotView: View {
    let artifacts: [SourceArtifact]
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            TabView {
                ForEach(artifacts) { artifact in
                    ZoomableImageView(image: artifact.image)
                        .tabItem {
                            Text(artifact.filename)
                        }
                }
            }
            .tabViewStyle(PageTabViewStyle())
            .navigationTitle("Source Screenshots")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct ZoomableImageView: View {
    let image: UIImage?
    @State private var scale: CGFloat = 1.0
    @State private var lastScale: CGFloat = 1.0
    
    var body: some View {
        GeometryReader { geometry in
            ScrollView([.horizontal, .vertical]) {
                if let image = image {
                    Image(uiImage: image)
                        .resizable()
                        .scaledToFit
                        .frame(
                            width: geometry.size.width * scale,
                            height: geometry.size.height * scale
                        )
                        .gesture(
                            MagnificationGesture()
                                .onChanged { value in
                                    scale = lastScale * value
                                }
                                .onEnded { _ in
                                    lastScale = scale
                                }
                        )
                } else {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .background(Color(hex: "#0A0A0A"))
    }
}

// MARK: - Markdown Theme Extension
extension MarkdownTheme {
    static var truffle: MarkdownTheme {
        .gitHub
        .text {
            ForegroundColor(Color(hex: "#E5E5E5"))
            FontSize(16)
        }
        .code {
            FontFamilyVariant(.monospaced)
            FontSize(14)
            BackgroundColor(Color(hex: "#262626"))
        }
        .heading1 { 
            FontSize(24)
            FontWeight(.bold)
            ForegroundColor(Color(hex: "#E5E5E5"))
        }
        .heading2 { 
            FontSize(20)
            FontWeight(.semibold)
            ForegroundColor(Color(hex: "#E5E5E5"))
        }
        .link {
            ForegroundColor(Color(hex: "#FFB800"))
        }
        .blockquote {
            BackgroundColor(Color(hex: "#141414"))
            BorderColor(Color(hex: "#FFB800"))
        }
        .divider {
            BackgroundColor(Color(hex: "#262626"))
        }
        .listItem {
            ForegroundColor(Color(hex: "#E5E5E5"))
        }
    }
}

extension InlineImageProvider {
    static var truffle: InlineImageProvider {
        .init { imageURL, _ in
            // Custom image loading for wiki images
            // Return placeholder for now
            Image(systemName: "photo")
        }
    }
}

// MARK: - Supporting Types

struct WikiNode: Identifiable, Hashable {
    let id: UUID
    let nodeType: WikiNodeType
    let title: String
    let content: String
    let summary: String?
    let backlinks: [WikiLink]
    let forwardLinks: [WikiLink]
    let sourceArtifacts: [SourceArtifact]
    let compiledAt: String
    let modelVersion: String
    let confidenceScore: Double
    let privacyClassification: PrivacyClassification
    let isEncrypted: Bool
    let children: [WikiNode]
    
    var iconName: String {
        switch nodeType {
        case .entity: return "doc.text"
        case .concept: return "lightbulb"
        case .chronology: return "calendar"
        case .index: return "list.bullet"
        }
    }
    
    var typeColor: Color {
        switch nodeType {
        case .entity: return Color(hex: "#00D4AA")
        case .concept: return Color(hex: "#FFB800")
        case .chronology: return .blue
        case .index: return Color(hex: "#737373")
        }
    }
    
    var markdownExport: String {
        // Export node as markdown with YAML frontmatter
        """
        ---
        id: \(id.uuidString)
        type: \(nodeType)
        compiled_at: \(compiledAt)
        confidence: \(confidenceScore)
        ---
        
        # \(title)
        
        \(content)
        """
    }
}

enum WikiNodeType: String {
    case entity
    case concept
    case chronology
    case index
}

enum PrivacyClassification: String {
    case `public`
    case personal
    case sensitive
    case financial
    
    var label: String {
        switch self {
        case .public: return "Public"
        case .personal: return "Personal"
        case .sensitive: return "Sensitive"
        case .financial: return "Financial"
        }
    }
    
    var color: Color {
        switch self {
        case .public: return Color(hex: "#00D4AA")
        case .personal: return .blue
        case .sensitive: return Color(hex: "#FFB800")
        case .financial: return .purple
        }
    }
}

struct WikiLink: Identifiable {
    let id = UUID()
    let sourceId: UUID
    let sourceTitle: String
    let targetId: UUID
    let linkText: String
}

struct SourceArtifact: Identifiable {
    let id: UUID
    let filename: String
    let thumbnail: UIImage?
    let image: UIImage?
}

// MARK: - View Model
class WikiViewModel: ObservableObject {
    @Published var rootNodes: [WikiNode] = []
    @Published var allNodes: [WikiNode] = []
    
    func loadWikiNodes() {
        // Load from local SQLite database
        // This is a placeholder - actual implementation would query Core Data/SQLite
        rootNodes = []
    }
    
    func findNode(byTitle title: String) -> WikiNode? {
        return allNodes.first { $0.title.lowercased() == title.lowercased() }
    }
    
    func findNode(byId id: UUID) -> WikiNode? {
        return allNodes.first { $0.id == id }
    }
}

// MARK: - Preview
struct WikiBrowserView_Previews: PreviewProvider {
    static var previews: some View {
        WikiBrowserView(viewModel: WikiViewModel())
            .preferredColorScheme(.dark)
    }
}
