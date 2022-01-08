use pyth_client::{
    load_mapping, load_price, load_product, CorpAction, PriceStatus, PriceType, PROD_HDR_SIZE,
};

fn main() {
    println!("Hello, world!");
    let url = "http://api.devnet.solana.com";
    let key = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
    let clnt = RpcClient::new(url.to_string());
    let mut akey = Pubkey::from_str(key).unwrap();
}
