use rspigot_core::plugins::Order;
use rspigot_plugin_protocol::v1;

use super::Malformed;

pub(super) fn order(order: v1::ServerOrder) -> Result<Order, Malformed> {
    match order.order {
        None => Err(Malformed::Empty),
        Some(_) => Err(Malformed::Unsupported("server order")),
    }
}
