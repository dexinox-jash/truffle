//
//  SearchView.swift
//  TruffleApp
//
//  Project Truffle - Search Interface
//  SPEC v2.0 Section 5.3.1: Mobile Companion - Search and navigation
//
//  Implements:
//  - Full-text search across wiki nodes
//  - Semantic search using embeddings
//  - Search filters and suggestions
//  - Recent searches
//

import SwiftUI
import Combine

// MARK: - Search View
struct SearchView: View {
    
    @ObservedObject var viewModel: SearchViewModel
    @Environment(\.dismiss) private var dismiss
    
    @State private var searchText = ""
    @State private var selectedFilter: SearchFilter = .all
    @State private var showFilters = false
    @FocusState private var isSearchFocused: Bool
    
    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Search Bar
                SearchBar(
                    text: $searchText,
                    isFocused: $isSearchFocused,
                    onClear: { searchText = "" }
                )
                .padding()
                
                // Filter Bar
                FilterBar(selectedFilter: $selectedFilter, showFilters: $showFilters)
                    .padding(.horizontal)
                
                // Search Results
                if searchText.isEmpty {
                    EmptySearchView(viewModel: viewModel)
                } else if viewModel.isSearching {
                    SearchingView()
                } else if viewModel.results.isEmpty {
                    NoResultsView(query: searchText)
                } else {
                    SearchResultsList(
                        results: viewModel.results,
                        onSelect: { result in
                            handleResultSelection(result)
                        }
                    )
                }
            }
            .background(Color(hex: "#0A0A0A"))
            .navigationTitle("Search")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onChange(of: searchText) { newValue in
                viewModel.search(query: newValue, filter: selectedFilter)
            }
            .onChange(of: selectedFilter) { newValue in
                if !searchText.isEmpty {
                    viewModel.search(query: searchText, filter: newValue)
                }
            }
            .onAppear {
                isSearchFocused = true
                viewModel.loadRecentSearches()
            }
        }
    }
    
    private func handleResultSelection(_ result: SearchResult) {
        // Navigate to selected result
        dismiss()
        // Post notification or use coordinator to navigate
        NotificationCenter.default.post(
            name: .init("NavigateToWikiNode"),
            object: result.nodeId
        )
    }
}

// MARK: - Search Bar
struct SearchBar: View {
    @Binding var text: String
    var isFocused: FocusState<Bool>.Binding
    let onClear: () -> Void
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "magnifyingglass")
                .foregroundColor(Color(hex: "#737373"))
            
            TextField("Search wiki...", text: $text)
                .foregroundColor(Color(hex: "#E5E5E5"))
                .focused(isFocused)
                .autocapitalization(.none)
                .disableAutocorrection(true)
            
            if !text.isEmpty {
                Button(action: onClear) {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(Color(hex: "#737373"))
                }
            }
        }
        .padding()
        .background(Color(hex: "#141414"))
        .cornerRadius(10)
    }
}

// MARK: - Filter Bar
struct FilterBar: View {
    @Binding var selectedFilter: SearchFilter
    @Binding var showFilters: Bool
    
    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(SearchFilter.allCases) { filter in
                    FilterChip(
                        filter: filter,
                        isSelected: selectedFilter == filter,
                        onTap: { selectedFilter = filter }
                    )
                }
            }
        }
    }
}

struct FilterChip: View {
    let filter: SearchFilter
    let isSelected: Bool
    let onTap: () -> Void
    
    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 4) {
                Image(systemName: filter.icon)
                    .font(.system(size: 12))
                Text(filter.label)
                    .font(.system(size: 13))
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(isSelected ? Color(hex: "#FFB800") : Color(hex: "#262626"))
            .foregroundColor(isSelected ? Color(hex: "#0A0A0A") : Color(hex: "#E5E5E5"))
            .cornerRadius(16)
        }
    }
}

// MARK: - Empty Search View
struct EmptySearchView: View {
    @ObservedObject var viewModel: SearchViewModel
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                // Recent Searches
                if !viewModel.recentSearches.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            Text("Recent")
                                .font(.system(size: 16, weight: .semibold))
                                .foregroundColor(Color(hex: "#E5E5E5"))
                            
                            Spacer()
                            
                            Button("Clear") {
                                viewModel.clearRecentSearches()
                            }
                            .font(.system(size: 14))
                            .foregroundColor(Color(hex: "#FFB800"))
                        }
                        
                        ForEach(viewModel.recentSearches, id: \.self) { search in
                            Button(action: {
                                // Re-run search
                            }) {
                                HStack {
                                    Image(systemName: "clock.arrow.circlepath")
                                        .foregroundColor(Color(hex: "#737373"))
                                    
                                    Text(search)
                                        .foregroundColor(Color(hex: "#E5E5E5"))
                                    
                                    Spacer()
                                }
                                .padding(.vertical, 8)
                            }
                        }
                    }
                }
                
                // Suggested Searches
                VStack(alignment: .leading, spacing: 12) {
                    Text("Suggested")
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundColor(Color(hex: "#E5E5E5"))
                    
                    FlowLayout(spacing: 8) {
                        ForEach(viewModel.suggestedSearches, id: \.self) { suggestion in
                            Button(action: {
                                // Run suggested search
                            }) {
                                Text(suggestion)
                                    .font(.system(size: 13))
                                    .padding(.horizontal, 12)
                                    .padding(.vertical, 6)
                                    .background(Color(hex: "#262626"))
                                    .foregroundColor(Color(hex: "#E5E5E5"))
                                    .cornerRadius(16)
                            }
                        }
                    }
                }
                
                // Search Tips
                VStack(alignment: .leading, spacing: 12) {
                    Text("Search Tips")
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundColor(Color(hex: "#E5E5E5"))
                    
                    VStack(alignment: .leading, spacing: 8) {
                        SearchTip(icon: "text.quote", text: "Use quotes for exact phrases")
                        SearchTip(icon: "tag", text: "Filter by type: type:receipt")
                        SearchTip(icon: "calendar", text: "Date range: after:2024-01-01")
                        SearchTip(icon: "link", text: "Find orphans: -has:backlinks")
                    }
                }
            }
            .padding()
        }
    }
}

struct SearchTip: View {
    let icon: String
    let text: String
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .foregroundColor(Color(hex: "#FFB800"))
                .frame(width: 20)
            
            Text(text)
                .font(.system(size: 14))
                .foregroundColor(Color(hex: "#737373"))
        }
    }
}

// MARK: - Searching View
struct SearchingView: View {
    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            
            ProgressView()
                .progressViewStyle(CircularProgressViewStyle(tint: Color(hex: "#FFB800")))
                .scaleEffect(1.5)
            
            Text("Searching...")
                .font(.system(size: 16))
                .foregroundColor(Color(hex: "#737373"))
            
            Spacer()
        }
    }
}

// MARK: - No Results View
struct NoResultsView: View {
    let query: String
    
    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            
            Image(systemName: "magnifyingglass.circle")
                .font(.system(size: 64))
                .foregroundColor(Color(hex: "#737373"))
            
            Text("No results for \"\(query)\"")
                .font(.system(size: 18, weight: .medium))
                .foregroundColor(Color(hex: "#E5E5E5"))
            
            Text("Try different keywords or check your spelling")
                .font(.system(size: 14))
                .foregroundColor(Color(hex: "#737373"))
                .multilineTextAlignment(.center)
            
            Spacer()
        }
        .padding()
    }
}

// MARK: - Search Results List
struct SearchResultsList: View {
    let results: [SearchResult]
    let onSelect: (SearchResult) -> Void
    
    var body: some View {
        List {
            ForEach(results) { result in
                SearchResultRow(result: result)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        onSelect(result)
                    }
            }
        }
        .listStyle(PlainListStyle())
        .background(Color(hex: "#0A0A0A"))
    }
}

struct SearchResultRow: View {
    let result: SearchResult
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: result.iconName)
                    .foregroundColor(result.typeColor)
                
                Text(result.title)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundColor(Color(hex: "#E5E5E5"))
                
                Spacer()
                
                // Relevance score indicator
                RelevanceIndicator(score: result.relevanceScore)
            }
            
            // Context snippet with highlighted matches
            if let snippet = result.snippet {
                HighlightedText(text: snippet, highlights: result.matchRanges)
                    .font(.system(size: 14))
                    .foregroundColor(Color(hex: "#737373"))
                    .lineLimit(2)
            }
            
            // Metadata
            HStack(spacing: 12) {
                Text(result.nodeType.label)
                    .font(.system(size: 11))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(result.typeColor.opacity(0.2))
                    .foregroundColor(result.typeColor)
                    .cornerRadius(4)
                
                if let date = result.lastModified {
                    Text(date)
                        .font(.system(size: 11))
                        .foregroundColor(Color(hex: "#737373"))
                }
                
                Spacer()
                
                // Backlinks count
                if result.backlinkCount > 0 {
                    HStack(spacing: 4) {
                        Image(systemName: "link")
                            .font(.system(size: 10))
                        Text("\(result.backlinkCount)")
                            .font(.system(size: 11))
                    }
                    .foregroundColor(Color(hex: "#737373"))
                }
            }
        }
        .padding(.vertical, 8)
        .background(Color(hex: "#141414"))
    }
}

struct RelevanceIndicator: View {
    let score: Double
    
    var body: some View {
        HStack(spacing: 2) {
            ForEach(0..<3) { i in
                Rectangle()
                    .fill(score > Double(i) * 0.33 ? Color(hex: "#00D4AA") : Color(hex: "#262626"))
                    .frame(width: 4, height: 8)
                    .cornerRadius(1)
            }
        }
    }
}

struct HighlightedText: View {
    let text: String
    let highlights: [NSRange]
    
    var body: some View {
        // Simplified highlighting - actual implementation would parse ranges
        Text(text)
    }
}

// MARK: - Flow Layout
struct FlowLayout: Layout {
    var spacing: CGFloat = 8
    
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = FlowResult(in: proposal.width ?? 0, subviews: subviews, spacing: spacing)
        return result.size
    }
    
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = FlowResult(in: bounds.width, subviews: subviews, spacing: spacing)
        for (index, subview) in subviews.enumerated() {
            subview.place(at: CGPoint(x: bounds.minX + result.positions[index].x,
                                      y: bounds.minY + result.positions[index].y),
                         proposal: .unspecified)
        }
    }
    
    struct FlowResult {
        var size: CGSize = .zero
        var positions: [CGPoint] = []
        
        init(in maxWidth: CGFloat, subviews: Subviews, spacing: CGFloat) {
            var x: CGFloat = 0
            var y: CGFloat = 0
            var lineHeight: CGFloat = 0
            
            for subview in subviews {
                let size = subview.sizeThatFits(.unspecified)
                
                if x + size.width > maxWidth && x > 0 {
                    x = 0
                    y += lineHeight + spacing
                    lineHeight = 0
                }
                
                positions.append(CGPoint(x: x, y: y))
                lineHeight = max(lineHeight, size.height)
                x += size.width + spacing
            }
            
            self.size = CGSize(width: maxWidth, height: y + lineHeight)
        }
    }
}

// MARK: - Search Types

enum SearchFilter: String, CaseIterable, Identifiable {
    case all
    case entity
    case concept
    case chronology
    case recent
    
    var id: String { rawValue }
    
    var label: String {
        switch self {
        case .all: return "All"
        case .entity: return "Entities"
        case .concept: return "Concepts"
        case .chronology: return "Dates"
        case .recent: return "Recent"
        }
    }
    
    var icon: String {
        switch self {
        case .all: return "doc.text.magnifyingglass"
        case .entity: return "doc.text"
        case .concept: return "lightbulb"
        case .chronology: return "calendar"
        case .recent: return "clock"
        }
    }
}

struct SearchResult: Identifiable {
    let id = UUID()
    let nodeId: UUID
    let title: String
    let snippet: String?
    let matchRanges: [NSRange]
    let nodeType: WikiNodeType
    let relevanceScore: Double
    let lastModified: String?
    let backlinkCount: Int
    
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
}

// MARK: - Search View Model
class SearchViewModel: ObservableObject {
    @Published var results: [SearchResult] = []
    @Published var isSearching = false
    @Published var recentSearches: [String] = []
    @Published var suggestedSearches: [String] = []
    
    private var searchCancellable: AnyCancellable?
    private let wikiViewModel: WikiViewModel
    private let searchQueue = DispatchQueue(label: "com.truffle.search", qos: .userInitiated)
    
    init(wikiViewModel: WikiViewModel) {
        self.wikiViewModel = wikiViewModel
        self.suggestedSearches = [
            "receipts",
            "travel",
            "contacts",
            "expenses",
            "ideas"
        ]
    }
    
    func search(query: String, filter: SearchFilter) {
        guard !query.isEmpty else {
            results = []
            return
        }
        
        isSearching = true
        
        // Debounce search
        searchCancellable?.cancel()
        searchCancellable = Just(query)
            .delay(for: .milliseconds(300), scheduler: searchQueue)
            .sink { [weak self] _ in
                self?.performSearch(query: query, filter: filter)
            }
    }
    
    private func performSearch(query: String, filter: SearchFilter) {
        // Full-text search using SQLite FTS
        // Semantic search using embeddings
        // Combine and rank results
        
        // Placeholder implementation
        let searchResults: [SearchResult] = []
        
        DispatchQueue.main.async { [weak self] in
            self?.results = searchResults
            self?.isSearching = false
            
            // Save to recent searches
            if !searchResults.isEmpty {
                self?.addToRecentSearches(query)
            }
        }
    }
    
    func loadRecentSearches() {
        // Load from UserDefaults
        recentSearches = UserDefaults.standard.stringArray(forKey: "recentSearches") ?? []
    }
    
    func addToRecentSearches(_ query: String) {
        var searches = recentSearches
        searches.removeAll { $0 == query }
        searches.insert(query, at: 0)
        searches = Array(searches.prefix(10))
        
        recentSearches = searches
        UserDefaults.standard.set(searches, forKey: "recentSearches")
    }
    
    func clearRecentSearches() {
        recentSearches = []
        UserDefaults.standard.removeObject(forKey: "recentSearches")
    }
}

// MARK: - Preview
struct SearchView_Previews: PreviewProvider {
    static var previews: some View {
        SearchView(viewModel: SearchViewModel(wikiViewModel: WikiViewModel()))
            .preferredColorScheme(.dark)
    }
}
