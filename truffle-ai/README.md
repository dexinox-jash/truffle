# Truffle AI - On-Device Multimodal Processing

On-device AI processing pipeline for Project Truffle using Gemma 4 E2B via llama.cpp.

## Overview

This crate provides the AI processing capabilities for Project Truffle, implementing:

- **On-device inference** - No cloud AI APIs, all processing happens locally
- **Gemma 4 E2B model** - 2B parameters, Q4_K_M quantized (~1.3GB)
- **llama.cpp integration** - Commit b2699 with Metal/CUDA/Vulkan backends
- **Content safety filtering** - MiniLM-based classifier for NSFW/medical/banking
- **JSON mode enforcement** - Structured output only, no free-text generation
- **Schema-based compilation** - Rules-driven knowledge extraction

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRUFFLE AI PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   Schema    │───▶│  Compiler   │───▶│   WikiNode Output   │ │
│  │   Loader    │    │   Engine    │    │   (JSON/MD)         │ │
│  └─────────────┘    └──────┬──────┘    └─────────────────────┘ │
│                            │                                    │
│         ┌──────────────────┼──────────────────┐                │
│         ▼                  ▼                  ▼                │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   Prompt    │    │   Gemma 4   │    │   Safety    │        │
│  │  Templates  │    │    E2B      │    │  Classifier │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## Key Constraints

| Constraint | Value | Specification |
|------------|-------|---------------|
| Processing | On-device only | Section 1.2 |
| Model | Gemma-4-2b-it-Q4_K_M.gguf | Section 2.2.2 |
| Timeout | 30 seconds | Section 2.2.2 |
| RAM Cap | 4GB | Section 2.2.2 |
| Output | JSON mode only | Section 4.2 |
| llama.cpp | Commit b2699 | Section 2.2.2 |

## Usage

```rust
use truffle_ai::{AiPipeline, PipelineConfig, CompilationOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the pipeline
    let config = PipelineConfig::default();
    let pipeline = AiPipeline::new(config).await?;
    
    // Compile a screenshot
    let options = CompilationOptions::default();
    let result = pipeline.compile_image("screenshot.png", options).await?;
    
    println!("Compiled: {:?}", result.wiki_node);
    
    Ok(())
}
```

## Module Structure

### `model/` - Model Management
- `manager.rs` - Download, cache, delta updates, rollback
- `gemma4.rs` - Gemma 4 E2B inference engine
- `verifier.rs` - SHA256 verification on startup

### `pipeline/` - Processing Pipeline
- `compiler.rs` - Main compilation orchestration
- `prompt.rs` - Versioned prompt templates with A/B testing
- `parser.rs` - JSON validation and normalization
- `safety.rs` - Content classifier (MiniLM)

### `schema/` - Schema-Based Compilation
- `loader.rs` - schema.md parser
- `matcher.rs` - Trigger matching with fuzzy logic
- `extractor.rs` - Field extraction with normalization

## Prompt Templates

Versioned prompt templates in `prompts/`:

- `system.txt` - Main system prompt with schema rules
- `receipt.txt` - Receipt extraction
- `contact.txt` - Contact card extraction
- `travel.txt` - Travel itinerary extraction

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test model_tests
cargo test --test safety_tests
cargo test --test schema_tests

# Run with logging
RUST_LOG=truffle_ai=debug cargo test
```

## Features

- `metal` - macOS Metal backend (default)
- `cuda` - NVIDIA CUDA backend
- `vulkan` - Vulkan backend for Linux/Windows
- `rocm` - AMD ROCm backend

## License

AGPL-3.0 - See LICENSE file for details.
