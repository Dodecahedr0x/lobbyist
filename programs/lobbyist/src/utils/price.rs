use pyth_min::price_update::Price;

// (price ± conf)* 10^exponent
pub fn price_to_u64(price: Price, decimals: u8) -> u64 {
    let base_price = price.price as u128 * 10_u128.pow(decimals as u32);
    let price = if price.exponent > 0 {
        base_price * 10_u128.pow(price.exponent as u32)
    } else if price.exponent < 0 {
        base_price / 10_u128.pow((-price.exponent) as u32)
    } else {
        base_price
    };
    price as u64
}
