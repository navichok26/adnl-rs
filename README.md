# ADNL

Minimal ADNL implementation in Rust (client-server only, without p2p for now). Specification of ADNL is available [here](https://github.com/tonstack/ton-docs/blob/main/ADNL/README.md).

| Feature       | Status                           |
| ------------- | -------------------------------- |
| ADNL Client   | ✅ Implemented                  |
| ADNL Server   | ❌ Not implemented              |
| ADNL P2P      | ❌ Not implemented              |
| async         | ❌ Not implemented              |
| ed25519 libs  | curve25519_dalek + x25519_dalek  |

## Quickstart
Run this example: `cargo run --example time --features "std dalek"`

```rust
use adnl::{AdnlBuilder, AdnlClient};
use std::error::Error;
use std::net::{SocketAddrV4, TcpStream};
use x25519_dalek::StaticSecret;

pub fn connect(
    ls_public: &str,
    ls_ip: &str,
    ls_port: u16,
) -> Result<AdnlClient<TcpStream>, Box<dyn Error>> {
    // decode liteserver public key
    let remote_public: [u8; 32] = base64::decode(ls_public)?
        .try_into()
        .map_err(|_| "bad public key length")?;

    // generate private key
    let local_secret = StaticSecret::new(rand::rngs::OsRng);

    // use TcpStream as a transport for our ADNL connection
    let transport = TcpStream::connect(SocketAddrV4::new(ls_ip.parse()?, ls_port))?;

    // build handshake using random session keys, encrypt it with ECDH(local_secret, remote_public)
    // then perform handshake over our TcpStream
    let client = AdnlBuilder::with_random_aes_params(&mut rand::rngs::OsRng)
        .perform_ecdh(local_secret, remote_public)
        .perform_handshake(transport)
        .map_err(|e| format!("{:?}", e))?;
    Ok(client)
}

fn main() -> Result<(), Box<dyn Error>> {
    // create AdnlClient
    let mut client = connect(
        "JhXt7H1dZTgxQTIyGiYV4f9VUARuDxFl/1kVBjLSMB8=",
        "65.21.74.140",
        46427,
    )?;

    // already serialized TL with gettime query
    let mut query = hex::decode("7af98bb435263e6c95d6fecb497dfd0aa5f031e7d412986b5ce720496db512052e8f2d100cdf068c7904345aad16000000000000")?;

    // send over ADNL, use random nonce
    client
        .send(&mut query, &mut rand::random())
        .map_err(|e| format!("{:?}", e))?;

    // receive result into vector, use 8192 bytes buffer
    let mut result = Vec::<u8>::new();
    client
        .receive::<_, 8192>(&mut result)
        .map_err(|e| format!("{:?}", e))?;

    // get time from serialized TL answer
    println!(
        "received: {}",
        u32::from_le_bytes(result[result.len() - 7..result.len() - 3].try_into()?)
    );
    Ok(())
}
```
