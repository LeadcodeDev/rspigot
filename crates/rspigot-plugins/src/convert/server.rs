use rspigot_core::admin::ListEntry;
use rspigot_core::plugins::{Order, ServerOrder, ServerState};
use rspigot_plugin_protocol::v1::{self, server_order::Order as Wire};

use super::Malformed;
use super::shared::required;

pub(super) fn order(order: v1::ServerOrder) -> Result<Order, Malformed> {
    let order = match required(order.order)? {
        Wire::Broadcast(text) => ServerOrder::Broadcast(text),
        Wire::DispatchCommand(line) => ServerOrder::DispatchCommand(line),
        Wire::Shutdown(_) => ServerOrder::Shutdown,
        Wire::Save(_) => ServerOrder::Save,
        Wire::SetWhitelistEnabled(enabled) => ServerOrder::SetWhitelistEnabled(enabled),
        Wire::WhitelistAdd(name) => ServerOrder::WhitelistAdd(name),
        Wire::WhitelistRemove(name) => ServerOrder::WhitelistRemove(name),
        Wire::OperatorAdd(name) => ServerOrder::OperatorAdd(name),
        Wire::OperatorRemove(name) => ServerOrder::OperatorRemove(name),
        Wire::BanAdd(_)
        | Wire::BanRemove(_)
        | Wire::IpBanAdd(_)
        | Wire::IpBanRemove(_)
        | Wire::CreateInventory(_) => return Err(Malformed::Unsupported("server order")),
    };
    Ok(Order::Server(order))
}

pub(super) fn wire_state(state: ServerState) -> v1::ServerState {
    let [tps_1m, tps_5m, tps_15m] = state.tps;
    v1::ServerState {
        max_players: wire_count(state.max_players),
        online: wire_count(state.online),
        tps_1m,
        tps_5m,
        tps_15m,
        whitelist_enabled: state.whitelist_enabled,
    }
}

pub(super) fn wire_names(names: Vec<ListEntry>) -> v1::ListedPlayers {
    v1::ListedPlayers {
        players: names
            .into_iter()
            .map(|entry| v1::ListedPlayer {
                name: entry.name,
                uuid: entry.uuid.map(|uuid| uuid.hyphenated().to_string()),
            })
            .collect(),
    }
}

fn wire_count(count: u32) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use rspigot_core::admin::ListEntry;
    use rspigot_core::plugins::{
        Answer, FromPlugin, Order, Question, ServerOrder, ServerState, ToPlugin,
    };
    use rspigot_plugin_protocol::v1::{self, answer, server_order};
    use uuid::Uuid;

    use super::*;
    use crate::convert::{from_plugin, question, to_plugin};

    fn convert(order: Option<server_order::Order>) -> Result<FromPlugin, Malformed> {
        from_plugin(v1::FromPlugin {
            body: Some(from_plugin::Body::Server(v1::ServerOrder { order })),
        })
    }

    fn server(order: ServerOrder) -> FromPlugin {
        FromPlugin::Order(Order::Server(order))
    }

    fn ask(asked: question::Question) -> Question {
        let converted = from_plugin(v1::FromPlugin {
            body: Some(from_plugin::Body::Question(v1::Question {
                request: 3,
                question: Some(asked),
            })),
        });
        let Ok(FromPlugin::Question {
            request: 3,
            question,
        }) = converted
        else {
            panic!("expected a question");
        };
        question
    }

    fn answered(answer: Answer) -> answer::Body {
        let Some(to_plugin::Body::Answer(v1::Answer {
            request: 3,
            body: Some(body),
        })) = to_plugin(ToPlugin::Answer { request: 3, answer }).body
        else {
            panic!("expected an answer");
        };
        body
    }

    #[test]
    fn server_orders_and_answers_convert() {
        assert_eq!(
            convert(Some(server_order::Order::Broadcast("§eHi".into()))).unwrap(),
            server(ServerOrder::Broadcast("§eHi".into()))
        );
        assert_eq!(
            convert(Some(server_order::Order::WhitelistAdd("Carol".into()))).unwrap(),
            server(ServerOrder::WhitelistAdd("Carol".into()))
        );
        assert_eq!(
            convert(Some(server_order::Order::Shutdown(v1::Empty {}))).unwrap(),
            server(ServerOrder::Shutdown)
        );
        assert!(matches!(convert(None), Err(Malformed::Empty)));
        assert!(matches!(
            convert(Some(server_order::Order::BanRemove("Steve".into()))),
            Err(Malformed::Unsupported("server order"))
        ));

        assert_eq!(
            ask(question::Question::Server(v1::Empty {})),
            Question::Server
        );
        assert_eq!(
            ask(question::Question::Whitelist(v1::Empty {})),
            Question::Whitelist
        );
        assert_eq!(
            ask(question::Question::Operators(v1::Empty {})),
            Question::Operators
        );
        assert_eq!(
            ask(question::Question::PlayerNamed("alice".into())),
            Question::PlayerNamed("alice".into())
        );

        assert_eq!(
            answered(Answer::Server(ServerState {
                max_players: 20,
                online: 2,
                tps: [19.5, 19.8, 20.0],
                whitelist_enabled: true,
            })),
            answer::Body::Server(v1::ServerState {
                max_players: 20,
                online: 2,
                tps_1m: 19.5,
                tps_5m: 19.8,
                tps_15m: 20.0,
                whitelist_enabled: true,
            })
        );
        let alice = ListEntry {
            name: "Alice".into(),
            uuid: Some(Uuid::from_u128(1)),
        };
        assert_eq!(
            answered(Answer::Names(vec![ListEntry::named("Carol"), alice])),
            answer::Body::Listed(v1::ListedPlayers {
                players: vec![
                    v1::ListedPlayer {
                        name: "Carol".into(),
                        uuid: None,
                    },
                    v1::ListedPlayer {
                        name: "Alice".into(),
                        uuid: Some("00000000-0000-0000-0000-000000000001".into()),
                    },
                ],
            })
        );
    }
}
