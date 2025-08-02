use std::ops::Range;

/// Range di priorità riservate per garantire ordine corretto
pub struct PriorityRanges;

impl PriorityRanges {
    /// Sistema critico - sempre per primo (security, compliance)
    pub const CRITICAL_SYSTEM: Range<i32> = 9000..10000;

    /// Policy globali di alto livello
    pub const GLOBAL_HIGH: Range<i32> = 8000..9000;

    /// Direttive ad alta priorità (@timeout, @if)
    pub const DIRECTIVE_HIGH: Range<i32> = 7000..8000;

    /// Policy globali normali
    pub const GLOBAL_NORMAL: Range<i32> = 5000..7000;

    /// Direttive normali (@parallel, @doc)
    pub const DIRECTIVE_NORMAL: Range<i32> = 3000..5000;

    /// Policy globali di supporto
    pub const GLOBAL_SUPPORT: Range<i32> = 1000..3000;

    /// Direttive di supporto (@log di debug)
    pub const DIRECTIVE_SUPPORT: Range<i32> = 500..1000;

    /// Monitoring e analytics - sempre per ultimo
    pub const MONITORING: Range<i32> = 0..500;
}
