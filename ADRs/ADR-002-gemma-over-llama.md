# ADR-002: Gemma 4 over Llama for On-Device AI

## Status

**Accepted** | Date: 2024-01-15 | Author: Systems Architect

## Context

Project Truffle's core innovation is on-device AI processing that never "phones home." We need a multimodal language model that can:

- Process screenshots (vision + language understanding)
- Extract structured information (receipts, contacts, travel)
- Generate markdown-formatted wiki content
- Run entirely on consumer hardware (8GB+ RAM devices)
- Operate without internet connectivity
- Comply with commercial licensing requirements

Two primary model families were evaluated:
1. **Google Gemma 4** (2B and 9B variants)
2. **Meta Llama 3.3** (8B and 70B variants)

## Decision

We will use **Google Gemma 4** (specifically the E2B 2B variant as default, with E4B 9B as premium option) for on-device AI processing.

## Consequences

### Positive

| Aspect | Gemma 4 Advantage |
|--------|-------------------|
| **License Safety** | Permissive commercial license (no revenue caps) |
| **Multimodal Native** | Built-in vision capabilities (no separate encoder) |
| **Size Efficiency** | 2B parameters = ~1.5GB quantized vs Llama 8B = ~5GB |
| **Google Backing** | Long-term support, regular updates |
| **Hardware Optimization** | Gemma optimized for consumer devices |
| **Context Length** | 128K tokens (E4B) vs Llama's 8K |
| **Safety Tuning** | Built-in content filtering capabilities |

### Negative

| Aspect | Gemma Challenge | Mitigation |
|--------|-----------------|------------|
| **Ecosystem** | Smaller than Llama's | Sufficient for our use case |
| **Community** | Less fine-tuning available | We do our own fine-tuning |
| **Vendor Lock-in** | Google's ecosystem | Model weights are portable |
| **Performance** | Slightly lower than Llama 70B | Acceptable for our latency targets |

## Alternatives Considered

### Llama 3.3

**Pros:**
- Larger community and ecosystem
- Extensive fine-tuned variants available
- Strong performance on benchmarks
- Meta's open approach

**Cons:**
- License restrictions (acceptable use policy)
- Not natively multimodal (requires CLIP integration)
- Larger model sizes (8B minimum viable)
- Potential license changes (historical precedent)

**Verdict:** Rejected due to licensing uncertainty and lack of native multimodal support.

### Phi-4 (Microsoft)

**Pros:**
- Excellent performance for size
- Strong vision capabilities

**Cons:**
- Restrictive license (research only initially)
- Smaller ecosystem than Gemma
- Uncertain long-term support

**Verdict:** Rejected due to licensing restrictions.

### Qwen-VL (Alibaba)

**Pros:**
- Strong multimodal performance
- Open weights

**Cons:**
- Export control concerns (China origin)
- Compliance complexity
- Limited English optimization

**Verdict:** Rejected due to compliance risks.

### Custom Model

**Pros:**
- Full control over architecture
- Optimized for our exact use case

**Cons:**
- Massive training cost ($500K+)
- Ongoing maintenance burden
- No pre-trained knowledge

**Verdict:** Rejected due to cost and time constraints.

## Implementation Details

### Model Configuration

```rust
// truffle-core/src/ai/gemma/config.rs

pub const DEFAULT_MODEL: &str = "gemma-4-2b-it-Q4_K_M.gguf";
pub const PREMIUM_MODEL: &str = "gemma-4-9b-it-Q4_K_M.gguf";

pub struct GemmaConfig {
    pub model_path: PathBuf,
    pub context_size: usize = 8192,      // 2B: 8K, 9B: 128K
    pub threads: usize = num_cpus::get(),
    pub gpu_layers: usize = 0,           // CPU default, Metal/CUDA optional
    pub batch_size: usize = 512,
    pub temperature: f32 = 0.7,
    pub top_p: f32 = 0.9,
    pub top_k: usize = 40,
}
```

### Runtime Integration

```rust
// llama.cpp wrapper configuration
pub struct LlamaCppConfig {
    pub commit_hash: &'static str = "b2699", // Pinned version
    pub backends: Vec<Backend>,
}

pub enum Backend {
    Metal,   // macOS
    Cuda,    // Windows/Linux with NVIDIA
    Vulkan,  // Linux/Windows fallback
    Cpu,     // Universal fallback
}
```

### Prompt Engineering

```markdown
<!-- truffle-core/src/ai/gemma/prompts/system.txt -->
You are a knowledge compilation assistant. Your task is to analyze 
screenshots and extract structured information according to the 
provided schema rules.

Output format: Valid JSON only, matching the WikiNode interface.
Do not include explanations or markdown formatting outside the JSON.

Schema rules:
{{schema_rules}}
```

```markdown
<!-- truffle-core/src/ai/gemma/prompts/compilation.txt -->
Compile this artifact according to schema rules. Extract entities, 
relationships, and temporal information. Output JSON only.

Image: [attached screenshot]
```

### Model Distribution

```yaml
# Model update configuration
model_distribution:
  source: huggingface
  cdn: cloudflare_r2
  verification:
    method: sha256
    checksum_required: true
  delta_updates:
    enabled: true
    algorithm: bsdiff
    average_size_reduction: "75%"
  rollback:
    keep_versions: 2
    auto_rollback_threshold: 5% hallucination increase
```

## Performance Characteristics

| Metric | Gemma 4 E2B (2B) | Gemma 4 E4B (9B) | Llama 3.3 (8B) |
|--------|------------------|------------------|----------------|
| Quantized Size | 1.5GB | 5.5GB | 4.5GB |
| RAM Usage | 2.5GB | 8GB | 6GB |
| Inference Speed* | 15 tokens/sec | 8 tokens/sec | 12 tokens/sec |
| Compilation Time** | 2-3s | 4-5s | 3-4s |
| Accuracy (Receipts) | 94% | 97% | 96% |
| License | Permissive | Permissive | Restrictive |

*On Apple M2, CPU only  
**End-to-end screenshot to wiki node

## A/B Testing Strategy

```rust
// Model variant testing
pub struct ModelExperiment {
    pub variant: ModelVariant,
    pub allocation: f32, // 0.0 - 1.0
    pub metrics: Vec<Metric>,
}

pub enum ModelVariant {
    Control,    // E2B (2B) - 90%
    Treatment,  // E4B (9B) - 10%
}

pub enum Metric {
    CompilationAccuracy,
    LatencyP50,
    LatencyP95,
    BatteryImpact,
    HallucinationRate,
}
```

Auto-promotion criteria:
- Accuracy improvement > 15%
- Latency increase < 2x
- No regression in battery life

## Fallback Strategy

If Gemma 4 becomes unsuitable:

1. **Immediate:** Switch to Llama 3.3 (license permitting)
2. **Short-term:** Evaluate newer Gemma versions
3. **Long-term:** Consider quantized proprietary models (with user consent)

All models implement the same `GemmaEngine` trait for swapability:

```rust
#[async_trait]
pub trait CompilationEngine: Send + Sync {
    async fn compile(&self, request: CompilationRequest) -> Result<CompilationResult, Error>;
    fn model_info(&self) -> ModelInfo;
}
```

## References

- [Gemma 4 Technical Report](https://ai.google.dev/gemma)
- [Llama 3.3 License](https://llama.meta.com/llama3/license/)
- [llama.cpp Documentation](https://github.com/ggerganov/llama.cpp)
- [Project Truffle Model Benchmarks](./benchmarks/model-performance.md)

---

**Decision Owner:** CTO  
**Stakeholders:** Engineering, Legal, Product  
**Review Date:** 2024-07-15 (6 months or Gemma 5 release)
