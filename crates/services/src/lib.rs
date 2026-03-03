#![no_std]

use sadas_sysapi::PrivacyLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Vault,
    PermissionBroker,
    Compositor,
    Shell,
    Sync,
    Updater,
    CompatibilityLayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServicePolicy {
    pub kind: ServiceKind,
    pub privacy: PrivacyLevel,
    pub network_allowed: bool,
    pub depends_on: Option<ServiceKind>,
}

pub fn default_policies() -> [ServicePolicy; 7] {
    [
        ServicePolicy {
            kind: ServiceKind::Vault,
            privacy: PrivacyLevel::Strict,
            network_allowed: false,
            depends_on: None,
        },
        ServicePolicy {
            kind: ServiceKind::PermissionBroker,
            privacy: PrivacyLevel::Strict,
            network_allowed: false,
            depends_on: Some(ServiceKind::Vault),
        },
        ServicePolicy {
            kind: ServiceKind::Compositor,
            privacy: PrivacyLevel::Standard,
            network_allowed: false,
            depends_on: Some(ServiceKind::PermissionBroker),
        },
        ServicePolicy {
            kind: ServiceKind::Shell,
            privacy: PrivacyLevel::Standard,
            network_allowed: false,
            depends_on: Some(ServiceKind::Compositor),
        },
        ServicePolicy {
            kind: ServiceKind::Sync,
            privacy: PrivacyLevel::Relaxed,
            network_allowed: true,
            depends_on: Some(ServiceKind::PermissionBroker),
        },
        ServicePolicy {
            kind: ServiceKind::Updater,
            privacy: PrivacyLevel::Standard,
            network_allowed: true,
            depends_on: Some(ServiceKind::PermissionBroker),
        },
        ServicePolicy {
            kind: ServiceKind::CompatibilityLayer,
            privacy: PrivacyLevel::Standard,
            network_allowed: false,
            depends_on: Some(ServiceKind::Shell),
        },
    ]
}
