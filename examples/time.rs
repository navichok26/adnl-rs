use adnl::{AdnlBuilder, AdnlClient};
use std::net::SocketAddrV4;
use x25519_dalek::StaticSecret;
use tokio::net::TcpStream;
use anyhow::{anyhow, Context, Result};

pub async fn connect(
    ls_public: &str,
    ls_ip: &str,
    ls_port: u16,
) -> Result<AdnlClient<TcpStream>> {
    // decode liteserver public key
    let remote_public: [u8; 32] = base64::decode(ls_public)
        .context("Error decode base64")?
        .try_into().map_err(|_| anyhow!("Bad public key length"))?;

    // generate private key
    let local_secret = StaticSecret::new(rand::rngs::OsRng);

    // use TcpStream as transport for our ADNL connection
    let transport = TcpStream::connect(SocketAddrV4::new(ls_ip.parse()?, ls_port)).await
        .context("Connection error")?;

    // build handshake using random session keys, encrypt it with ECDH(local_secret, remote_public)
    // then perform handshake over our TcpStream
    let client = AdnlBuilder::with_random_aes_params(&mut rand::rngs::OsRng)
        .perform_ecdh(local_secret, remote_public)
        .perform_handshake(transport).await?;
    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    // create AdnlClient
    let mut client = connect(
        "JhXt7H1dZTgxQTIyGiYV4f9VUARuDxFl/1kVBjLSMB8=",
        "65.21.74.140",
        46427,
    ).await?;

    // already serialized TL with gettime query
    let mut query = hex::decode("7af98bb435263e6c95d6fecb497dfd0aa5f031e7d412986b5ce720496db512052e8f2d100cdf068c7904345aad16000000000000")?;

    // send over ADNL, use random nonce
    client
        .send(&mut query, &mut rand::random()).await?;

    // receive result into vector, use 8192 bytes buffer
    let mut result = Vec::<u8>::new();
    client
        .receive::<_, 8192>(&mut result).await?;

    // get time from serialized TL answer
    println!(
        "received: {}",
        u32::from_le_bytes(result[result.len() - 7..result.len() - 3].try_into()?)
    );
    Ok(())
}
