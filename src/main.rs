use pyth_client::{
    load_mapping, load_price, load_product, CorpAction, Price, PriceStatus, PriceType, Product,
    PROD_HDR_SIZE,
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

struct Context<'a> {
    client: &'a RpcClient,
}

fn main() {
    let url = "http://api.devnet.solana.com";
    let client = RpcClient::new(url.to_string());

    // Account key of the first Pyth mapping account.
    // The Mapping accounts are organized as a linked list, so each Mapping account
    // points to the next Mapping accont.
    let key = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
    let mut mapping_account_key = Pubkey::from_str(key).unwrap();

    loop {
        let map_data = client.get_account_data(&mapping_account_key).unwrap();
        let map_acct = load_mapping(&map_data).unwrap();
        let ctx = Context { client: &client };
        let mut i = 0;

        // Each Mapping account contains a list of Product accounts.
        for product_account_key in &map_acct.products {
            let product_account_pkey = Pubkey::new(&product_account_key.val);
            let product_account_data = client.get_account_data(&product_account_pkey).unwrap();
            let product_account = load_product(&product_account_data).unwrap();
            println!("product_account .. {:?}", product_account_pkey);

            print_product_attrs(&product_account);
            if !product_account.px_acc.is_valid() {
                continue;
            }

            // Likewise, each Product account has a pointer to a linked list of 
            // Price accounts.
            let mut price_account_pkey = Pubkey::new(&product_account.px_acc.val);
            loop {
                let price_account_data = ctx.client.get_account_data(&price_account_pkey).unwrap();
                let price_account = load_price(&price_account_data).unwrap();
                println!("  price_account .. {:?}", price_account_pkey);
                print_price_info(&price_account);

                if price_account.next.is_valid() {
                    price_account_pkey = Pubkey::new(&price_account.next.val);
                } else {
                    break;
                }
            }

            i += 1;
            if i == map_acct.num {
                break;
            }
        }

        if !map_acct.next.is_valid() {
            break;
        }
        mapping_account_key = Pubkey::new(&map_acct.next.val);
    }
}

/**
 * Prints all the attributes of a Product account.
 */
fn print_product_attrs(product_account: &Product) {
    let mut product_size = product_account.size as usize - PROD_HDR_SIZE;
    // Source code has spread operator. But we don't need it to run the following code.
    // let mut product_attr_iterator = (&product_account.attr[..]).iter();
    let mut product_attr_iterator = (&product_account.attr).iter();
    while product_size > 0 {
        let key = get_attr_str(&mut product_attr_iterator);
        let val = get_attr_str(&mut product_attr_iterator);
        println!("  {:.<16} {}", key, val);
        product_size -= 2 + key.len() + val.len();
    }
}

/**
 * Prints all the price info in a Price account.
 */
fn print_price_info(price_account: &Price) {
    match price_account.get_current_price() {
        Some(p) => {
            println!("    price ........ {} x 10^{}", p.price, p.expo);
            println!("    conf ......... {} x 10^{}", p.conf, p.expo);
        }
        None => {
            println!("    price ........ unavailable");
            println!("    conf ......... unavailable");
        }
    }

    println!(
        "    price_type ... {}",
        get_price_type(&price_account.ptype)
    );
    println!("    exponent ..... {}", price_account.expo);
    println!(
        "    status ....... {}",
        get_status(&price_account.agg.status)
    );
    println!(
        "    corp_act ..... {}",
        get_corp_act(&price_account.agg.corp_act)
    );

    println!("    num_qt ....... {}", price_account.num_qt);
    println!("    valid_slot ... {}", price_account.valid_slot);

    match price_account.get_twap() {
        Some(twap) => {
            println!("    twap ......... {} x 10^{}", twap.price, twap.expo);
            println!("    twac ......... {} x 10^{}", twap.conf, twap.expo);
        }
        None => {
            println!("    twap ......... unavailable");
            println!("    twac ......... unavailable");
        }
    }
}

/**
 * Transforms an iterator over the byte array of a Product account's attribute 
 * into a string.
 */
fn get_attr_str<'a, T>(iterator: &mut T) -> String
where
    T: Iterator<Item = &'a u8>,
{
    // So the length is the first element of the section, u8 int.
    let mut len = *iterator.next().unwrap() as usize;
    // Initialize a string of that size.
    let mut val = String::with_capacity(len);
    while len > 0 {
        // Then keep pushing every additional byte as a character into the string.
        val.push(*iterator.next().unwrap() as char);
        len -= 1;
    }

    // Return the string.
    return val;
}

fn get_price_type(ptype: &PriceType) -> &'static str {
    match ptype {
        PriceType::Unknown => "unknown",
        PriceType::Price => "price",
    }
}

fn get_status(st: &PriceStatus) -> &'static str {
    match st {
        PriceStatus::Unknown => "unknown",
        PriceStatus::Trading => "trading",
        PriceStatus::Halted => "halted",
        PriceStatus::Auction => "auction",
    }
}

fn get_corp_act(cact: &CorpAction) -> &'static str {
    match cact {
        CorpAction::NoCorpAct => "nocorpact",
    }
}
