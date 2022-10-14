use gdnative::prelude::*;

use super::Battlescape;

#[derive(Debug, Eq, PartialEq)]
pub enum BattlescapeSignal {
    Poopi,
    Var(String),
}
impl BattlescapeSignal {
    const fn name(&self) -> &'static str {
        match self {
            Self::Poopi => "Poopi",
            Self::Var(_) => "Var",
        }
    }

    const fn params(&self) -> &[(&str, VariantType)] {
        match self {
            Self::Poopi => &[],
            Self::Var(_) => &[("param", VariantType::GodotString)],
        }
    }

    pub fn emit_signal(self, owner: &Node2D) {
        let signal = self.name();
        match self {
            Self::Poopi => owner.emit_signal(signal, &[]),
            Self::Var(s) => owner.emit_signal(signal, &[s.owned_to_variant()]),
        };
    }

    /// Create dummy signals to call `name()` and `params()` on them.
    fn _dummy() -> [Self; std::mem::variant_count::<Self>()] {
        [Self::Poopi, Self::Var(Default::default())]
    }

    /// Automaticaly register all signals.
    pub fn register_signal(builder: &ClassBuilder<Battlescape>) {
        for s in BattlescapeSignal::_dummy() {
            let mut b = builder.signal(s.name());
            for &(parameter_name, parameter_type) in s.params() {
                b = b.with_param(parameter_name, parameter_type)
            }
            b.done();
        }
    }
}
