#![no_std]

use sadas_sysapi::PrivacyLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Vault,
    PermissionBroker,
    Compositor,
    Shell,
    Sync,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServicePolicy {
    pub kind: ServiceKind,
    pub privacy: PrivacyLevel,
    pub network_allowed: bool,
}

pub fn default_policies() -> [ServicePolicy; 5] {
    [
        ServicePolicy {
            kind: ServiceKind::Vault,
            privacy: PrivacyLevel::Strict,
            network_allowed: false,
        },
        ServicePolicy {
            kind: ServiceKind::PermissionBroker,
            privacy: PrivacyLevel::Strict,
            network_allowed: false,
        },
        ServicePolicy {
            kind: ServiceKind::Compositor,
            privacy: PrivacyLevel::Standard,
            network_allowed: false,
        },
        ServicePolicy {
            kind: ServiceKind::Shell,
            privacy: PrivacyLevel::Standard,
            network_allowed: false,
        },
        ServicePolicy {
            kind: ServiceKind::Sync,
            privacy: PrivacyLevel::Relaxed,
            network_allowed: true,
        },
    ]
}
