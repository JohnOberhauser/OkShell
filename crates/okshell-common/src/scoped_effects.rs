use reactive_graph::effect::Effect;
use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Dispose;

#[derive(Clone, Debug)]
pub struct OwnedEffect(Effect<LocalStorage>);

impl OwnedEffect {
    pub fn new(f: impl Fn(Option<()>) + 'static) -> Self {
        Self(Effect::new(f))
    }
}

impl Drop for OwnedEffect {
    fn drop(&mut self) {
        self.0.dispose();
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectScope(Vec<OwnedEffect>);

impl EffectScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, f: impl Fn(Option<()>) + 'static) {
        self.0.push(OwnedEffect::new(f));
    }
}
