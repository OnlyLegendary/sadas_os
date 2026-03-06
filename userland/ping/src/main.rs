use sadas_net::{checksum16, dhcp_discover_stub, IcmpEcho, LoopbackNic, NetworkDevice};

fn main() {
    let mut nic = LoopbackNic::virtio_default();
    let lease = dhcp_discover_stub(&nic);

    let mut packet = [0u8; 64];
    let echo = IcmpEcho { id: 1, seq: 1 };
    let bytes = echo
        .encode_request(b"sadas-ping", &mut packet)
        .expect("icmp encode");

    nic.send_frame(&packet[..bytes]).expect("send frame");

    let mut rx = [0u8; 128];
    let got = nic.recv_frame(&mut rx).expect("recv frame");
    let csum = checksum16(&rx[..got]);

    println!(
        "PING 10.0.2.2 via {:?}: lease={}.{}.{}.{} payload={} checksum={:#06x}",
        nic.info().backend,
        lease.address.0[0],
        lease.address.0[1],
        lease.address.0[2],
        lease.address.0[3],
        got,
        csum
    );
}
