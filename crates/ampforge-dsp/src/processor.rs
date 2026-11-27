//! Contratos modulares do motor DSP (Spec 053).
//!
//! Define a fronteira estrita entre a thread de controle (Off-RT: criação,
//! `prepare`, carregamento de recursos, drenagem de `RetiredReaper`) e a
//! thread de áudio em tempo real (RT: `process`, `apply_param_rt`, `reset`,
//! `latency_samples`, `drain_retired` com zero alocações e zero locks).

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// Formato de canais declarado por um processador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    MonoToStereo,
}

/// Especificação de áudio entregue em `prepare()` (Off-RT).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioSpec {
    pub sample_rate: u32,
    pub max_block_size: usize,
    pub layout: ChannelLayout,
}

impl AudioSpec {
    pub fn new(sample_rate: u32, max_block_size: usize, layout: ChannelLayout) -> Result<Self, DspError> {
        if !matches!(sample_rate, 44_100 | 48_000 | 88_200 | 96_000) {
            return Err(DspError::InvalidSpec("Taxa de amostragem não suportada"));
        }
        if max_block_size == 0 || max_block_size > 8192 {
            return Err(DspError::InvalidSpec("Tamanho máximo de bloco inválido"));
        }
        Ok(Self {
            sample_rate,
            max_block_size,
            layout,
        })
    }
}

/// Identificador estável de módulo (ex.: `"amp.marshall.1959_super_lead"`).
pub type ModuleId = &'static str;

/// Identificador de revisão de circuito/modelo (ex.: `"1969-jmp-sl-a-pre-july"`).
pub type ModelRevision = &'static str;

/// Identificador numérico compacto de parâmetro para despacho O(1) na thread RT.
pub type ParamId = u32;

/// Curva de resposta física de potenciômetro / controle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamTaper {
    Linear,
    Log10,
    Audio25,
    Discrete,
    Boolean,
}

/// Valor tipado de parâmetro para atualização escalar O(1) sem alocação na thread RT.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamValue {
    Float(f32),
    Bool(bool),
    Enum(&'static str),
}

/// Descritor estático de parâmetro exposto pelo `ModelDescriptor`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamDescriptor {
    pub id: ParamId,
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub taper: ParamTaper,
    pub enum_variants: &'static [&'static str],
}

impl ParamDescriptor {
    /// Valida um `ParamValue` contra os limites e tipo declarados neste descritor.
    pub fn validate(&self, val: ParamValue) -> Result<(), ParamError> {
        match (self.taper, val) {
            (ParamTaper::Boolean, ParamValue::Bool(_)) => Ok(()),
            (ParamTaper::Discrete, ParamValue::Enum(s)) => {
                if self.enum_variants.is_empty() || self.enum_variants.contains(&s) {
                    Ok(())
                } else {
                    Err(ParamError::InvalidEnumVariant { key: self.key, variant: s })
                }
            }
            (ParamTaper::Discrete, ParamValue::Float(v)) => {
                if !v.is_finite() {
                    return Err(ParamError::NonFinite { key: self.key });
                }
                if v < self.min || v > self.max {
                    return Err(ParamError::OutOfRange {
                        key: self.key,
                        value: v,
                        min: self.min,
                        max: self.max,
                    });
                }
                Ok(())
            }
            (ParamTaper::Linear | ParamTaper::Log10 | ParamTaper::Audio25, ParamValue::Float(v)) => {
                if !v.is_finite() {
                    return Err(ParamError::NonFinite { key: self.key });
                }
                if v < self.min || v > self.max {
                    return Err(ParamError::OutOfRange {
                        key: self.key,
                        value: v,
                        min: self.min,
                        max: self.max,
                    });
                }
                Ok(())
            }
            _ => Err(ParamError::TypeMismatch { key: self.key }),
        }
    }
}

/// Erro de validação ou aplicação de parâmetro.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    UnknownParamKey(String),
    UnknownParamId(ParamId),
    OutOfRange {
        key: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
    NonFinite {
        key: &'static str,
    },
    InvalidEnumVariant {
        key: &'static str,
        variant: &'static str,
    },
    TypeMismatch {
        key: &'static str,
    },
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownParamKey(k) => write!(f, "E_VALIDATION: parâmetro desconhecido '{}'", k),
            Self::UnknownParamId(id) => write!(f, "E_VALIDATION: id de parâmetro desconhecido {}", id),
            Self::OutOfRange { key, value, min, max } => {
                write!(f, "E_VALIDATION: '{}'={} fora de [{}..={}]", key, value, min, max)
            }
            Self::NonFinite { key } => write!(f, "E_VALIDATION: '{}' não é finito (NaN/Inf)", key),
            Self::InvalidEnumVariant { key, variant } => {
                write!(f, "E_VALIDATION: variante '{}' inválida para '{}'", variant, key)
            }
            Self::TypeMismatch { key } => write!(f, "E_VALIDATION: tipo incompatível para '{}'", key),
        }
    }
}

impl std::error::Error for ParamError {}

/// Erro de infraestrutura DSP modular (Off-RT / contrato).
#[derive(Debug, Clone, PartialEq)]
pub enum DspError {
    InvalidSpec(&'static str),
    UnknownModel(String),
    NotPrepared,
    Param(ParamError),
}

impl fmt::Display for DspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpec(msg) => write!(f, "E_DSP_SPEC: {}", msg),
            Self::UnknownModel(id) => write!(f, "E_DSP_MODEL: modelo desconhecido '{}'", id),
            Self::NotPrepared => write!(f, "E_DSP_STATE: processador não preparado"),
            Self::Param(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for DspError {}

impl From<ParamError> for DspError {
    fn from(e: ParamError) -> Self {
        Self::Param(e)
    }
}

/// Bloco de áudio planar imutável entregue a `AudioProcessor::process`.
pub struct AudioBlockRef<'a> {
    channels: [&'a [f32]; 2],
    num_channels: usize,
    num_frames: usize,
}

impl<'a> AudioBlockRef<'a> {
    #[inline]
    pub fn from_mono(ch0: &'a [f32]) -> Self {
        Self {
            channels: [ch0, &[]],
            num_channels: 1,
            num_frames: ch0.len(),
        }
    }

    #[inline]
    pub fn from_stereo(ch0: &'a [f32], ch1: &'a [f32]) -> Self {
        let len = ch0.len().min(ch1.len());
        Self {
            channels: [&ch0[..len], &ch1[..len]],
            num_channels: 2,
            num_frames: len,
        }
    }

    #[inline]
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.num_frames
    }

    #[inline]
    pub fn channel(&self, idx: usize) -> &[f32] {
        if idx < self.num_channels {
            &self.channels[idx][..self.num_frames]
        } else {
            &self.channels[0][..self.num_frames]
        }
    }
}

/// Bloco de áudio planar mutável preenchido por `AudioProcessor::process`.
pub struct AudioBlockMut<'a> {
    ch0: &'a mut [f32],
    ch1: Option<&'a mut [f32]>,
    num_frames: usize,
}

impl<'a> AudioBlockMut<'a> {
    #[inline]
    pub fn from_mono(ch0: &'a mut [f32]) -> Self {
        let len = ch0.len();
        Self {
            ch0,
            ch1: None,
            num_frames: len,
        }
    }

    #[inline]
    pub fn from_stereo(ch0: &'a mut [f32], ch1: &'a mut [f32]) -> Self {
        let len = ch0.len().min(ch1.len());
        Self {
            ch0: &mut ch0[..len],
            ch1: Some(&mut ch1[..len]),
            num_frames: len,
        }
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.num_frames
    }

    #[inline]
    pub fn num_channels(&self) -> usize {
        if self.ch1.is_some() { 2 } else { 1 }
    }

    #[inline]
    pub fn channel_mut(&mut self, idx: usize) -> &mut [f32] {
        let n = self.num_frames;
        if idx == 1 {
            if let Some(ref mut c1) = self.ch1 {
                return &mut c1[..n];
            }
        }
        &mut self.ch0[..n]
    }
}

/// Contexto físico compartilhado entre nós adjacentes sem acoplamento direto de structs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessContext {
    pub sample_rate: f32,
    pub oversampling_factor: usize,
    pub input_impedance_ohms: f32,
    pub speaker_resonance_hz: f32,
    pub frame_count: usize,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            oversampling_factor: 2,
            input_impedance_ohms: 1_000_000.0,
            speaker_resonance_hz: 98.0,
            frame_count: 128,
        }
    }
}

/// Recurso pesado aposentado na thread RT para desalocação segura na thread de controle.
pub type RetiredResource = Box<dyn Any + Send>;

/// Coletor lock-free onde a thread RT deposita recursos aposentados sem chamar `drop`.
pub trait RetiredSink {
    fn push_retired(&mut self, resource: RetiredResource) -> Result<(), RetiredResource>;
}

/// Implementação simples de `RetiredSink` baseada em `SpscOwnedQueue` para drenagem off-RT.
pub struct SpscRetiredSink<const CAP: usize> {
    tx: std::sync::Arc<crate::queue::SpscOwnedQueue<RetiredResource, CAP>>,
}

impl<const CAP: usize> SpscRetiredSink<CAP> {
    pub fn new(tx: std::sync::Arc<crate::queue::SpscOwnedQueue<RetiredResource, CAP>>) -> Self {
        Self { tx }
    }
}

impl<const CAP: usize> RetiredSink for SpscRetiredSink<CAP> {
    #[inline]
    fn push_retired(&mut self, resource: RetiredResource) -> Result<(), RetiredResource> {
        self.tx.push(resource)
    }
}

/// Descritor estático completo de um modelo/módulo DSP registrado.
#[derive(Debug, Clone, Copy)]
pub struct ModelDescriptor {
    pub module_id: ModuleId,
    pub model_revision: ModelRevision,
    pub display_name: &'static str,
    pub layout: ChannelLayout,
    pub params: &'static [ParamDescriptor],
}

impl ModelDescriptor {
    /// Busca descritor de parâmetro por chave estável.
    #[inline]
    pub fn find_param(&self, key: &str) -> Option<&ParamDescriptor> {
        self.params.iter().find(|p| p.key == key)
    }

    /// Valida um lote inteiro de parâmetros de forma atômica contra este descritor.
    pub fn validate_batch<'a>(
        &self,
        batch: &[(&'a str, ParamValue)],
    ) -> Result<Vec<(ParamId, ParamValue)>, ParamError> {
        let mut resolved = Vec::with_capacity(batch.len());
        for &(key, val) in batch {
            let desc = self
                .params
                .iter()
                .find(|p| p.key == key)
                .ok_or_else(|| ParamError::UnknownParamKey(key.to_string()))?;
            desc.validate(val)?;
            resolved.push((desc.id, val));
        }
        Ok(resolved)
    }
}

/// Contrato comum de todos os blocos DSP modulares (Specs 053–092).
pub trait AudioProcessor: Send {
    /// Descritor estático do módulo e seus parâmetros.
    fn descriptor(&self) -> &ModelDescriptor;

    /// Preparação Off-RT: aloca buffers para `spec` e pré-computa tabelas.
    fn prepare(&mut self, spec: AudioSpec) -> Result<(), DspError>;

    /// Reinicializa estado efêmero em O(1) na thread RT (zero-alloc).
    fn reset(&mut self);

    /// Atualiza um parâmetro escalar/enum na thread RT em O(1) (zero-alloc).
    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError>;

    /// Processa um bloco de áudio na thread RT (zero-alloc, zero-lock, saída finita).
    fn process(
        &mut self,
        ctx: &ProcessContext,
        input: &AudioBlockRef<'_>,
        output: &mut AudioBlockMut<'_>,
    );

    /// Latência algorítmica do módulo em amostras na taxa preparada.
    fn latency_samples(&self) -> usize;

    /// Transfere recursos aposentados para `sink` sem desalocar na thread RT.
    fn drain_retired(&mut self, _sink: &mut dyn RetiredSink) {}
}

/// Função fábrica para instanciar um processador na thread de controle (Off-RT).
pub type ProcessorFactory = fn() -> Box<dyn AudioProcessor>;

/// Registro central de modelos DSP disponíveis (Off-RT).
#[derive(Default)]
pub struct ProcessorRegistry {
    entries: HashMap<&'static str, (ModelDescriptor, ProcessorFactory)>,
    aliases: HashMap<&'static str, &'static str>,
}

impl ProcessorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        descriptor: ModelDescriptor,
        factory: ProcessorFactory,
        aliases: &[&'static str],
    ) {
        let id = descriptor.module_id;
        self.entries.insert(id, (descriptor, factory));
        for &alias in aliases {
            self.aliases.insert(alias, id);
        }
    }

    pub fn resolve_id<'a>(&self, id_or_alias: &'a str) -> Option<&'static str> {
        if let Some((desc, _)) = self.entries.get(id_or_alias) {
            return Some(desc.module_id);
        }
        self.aliases.get(id_or_alias).copied()
    }

    pub fn descriptor(&self, id_or_alias: &str) -> Option<&ModelDescriptor> {
        let id = self.resolve_id(id_or_alias)?;
        self.entries.get(id).map(|(d, _)| d)
    }

    /// Instancia e prepara o processador na thread de controle (Off-RT).
    pub fn create(&self, id_or_alias: &str, spec: AudioSpec) -> Result<Box<dyn AudioProcessor>, DspError> {
        let id = self
            .resolve_id(id_or_alias)
            .ok_or_else(|| DspError::UnknownModel(id_or_alias.to_string()))?;
        let (_, factory) = self
            .entries
            .get(id)
            .ok_or_else(|| DspError::UnknownModel(id_or_alias.to_string()))?;
        let mut proc = factory();
        proc.prepare(spec)?;
        Ok(proc)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
