#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerSource {
    Apic,
    HpetFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerState {
    pub source: TimerSource,
    pub hz: u32,
    pub enabled: bool,
}

pub struct Timer {
    state: TimerState,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            state: TimerState {
                source: TimerSource::Apic,
                hz: 0,
                enabled: false,
            },
        }
    }

    pub fn start_apic_periodic(&mut self, hz: u32) {
        self.state = TimerState {
            source: TimerSource::Apic,
            hz,
            enabled: hz > 0,
        };
    }

    pub fn state(&self) -> TimerState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_apic_timer() {
        let mut t = Timer::new();
        t.start_apic_periodic(100);
        let s = t.state();
        assert_eq!(s.source, TimerSource::Apic);
        assert!(s.enabled);
    }
}
