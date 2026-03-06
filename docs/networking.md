# Networking (phase baseline)

This phase adds a pragmatic, device-first networking path:

- QEMU baseline device: `virtio-net`
- Real hardware target baseline: `Intel e1000e`

## Stack scope

Implemented crate: `crates/net`

- Ethernet frame encode/parse
- ARP packet encode
- IPv4 packet encode
- ICMP echo request generation (ping payload)
- DHCP lease bootstrap stub (for QEMU-like NAT defaults)
- Minimal TCP HTTP GET helper

## Userland tools

- `/bin/ping` model: `cargo run -p sadas-user-ping`
- `/bin/httpget` model: `cargo run -p sadas-user-httpget -- example.com /`

## Notes

Current implementation is a bring-up baseline for integration and testing in QEMU-hosted development.
Kernel-driver wiring (virtio-net MMIO/PCI and e1000e MMIO/PCI on hardware) will use this crate's packet and device abstractions.
