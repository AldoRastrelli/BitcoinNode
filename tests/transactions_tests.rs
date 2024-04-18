use rusteze::{
    message_structs::{
        compact_size::CompactSize, input::Input, outpoint::Outpoint, output::Output,
        tx_message::TXMessage,
    },
    node::{utxo_collector::UtxoCollector, wallets::wallet_handler::WalletHandler},
};

#[test]
fn test_wallet_flow_balance() {
    let mut wallet_handler = WalletHandler::default();

    wallet_handler.new_wallet((
        "test_name".to_string(),
        "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
    ));

    let mut utxo_collector = UtxoCollector::default();

    // Here we insert the utxos with the wallet's address
    let tx = get_transaction();
    utxo_collector.add_utxo(tx);
    let utxos = utxo_collector.clone().utxos;
    for utxo in utxos {
        utxo_collector
            .utxos
            .insert("mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc".to_string(), utxo.1);
    }

    wallet_handler.add_utxo_to_wallets(&utxo_collector);
    let wallet = wallet_handler.get_actual_wallet().unwrap();

    assert_eq!(wallet.get_balance(), 1084265);
}

#[test]
fn test_wallet_handler_flow_balance() {
    let mut wallet_handler = WalletHandler::default();

    wallet_handler.new_wallet((
        "test_name".to_string(),
        "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
    ));

    let mut utxo_collector = UtxoCollector::default();

    // Here we insert the utxos with the wallet's address
    let tx = get_transaction();
    utxo_collector.add_utxo(tx);
    let utxos = utxo_collector.clone().utxos;
    for utxo in utxos {
        utxo_collector
            .utxos
            .insert("mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc".to_string(), utxo.1);
    }

    wallet_handler.add_utxo_to_wallets(&utxo_collector);
    let wallet = wallet_handler.get_actual_wallet().unwrap();

    assert_eq!(wallet.get_balance(), wallet_handler.get_actual_balance());
}

#[test]
fn test_wallet_flow_transaction() {
    let mut wallet_handler = WalletHandler::default();

    wallet_handler.new_wallet((
        "test_name".to_string(),
        "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
    ));

    let mut utxo_collector = UtxoCollector::default();

    // Here we insert the utxos with the wallet's address
    let tx = get_transaction();
    utxo_collector.add_utxo(tx);
    let utxos = utxo_collector.clone().utxos;
    for utxo in utxos {
        utxo_collector
            .utxos
            .insert("mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc".to_string(), utxo.1);
    }

    wallet_handler.add_utxo_to_wallets(&utxo_collector);

    let tx = wallet_handler
        .create_transaction((
            "mhnNeQGhRKe28qGXZEvfAbWhWZ2ritjuZg".to_string(),
            "label".to_string(),
            5000,
            3000,
        ))
        .unwrap();

    println!("{:?}", tx);

    let inpu_len = tx.input_list.len();

    assert_eq!(inpu_len, 1);
}

fn get_transaction() -> TXMessage {
    TXMessage::new(
        2,
        CompactSize {
            prefix: 0,
            number_vec: vec![1],
            number: 1,
        },
        vec![Input::new(
            Outpoint::new(
                [
                    159, 134, 40, 40, 100, 228, 65, 170, 179, 148, 205, 50, 249, 242, 17, 0, 61,
                    203, 223, 193, 219, 7, 149, 22, 242, 238, 203, 35, 89, 127, 82, 201,
                ],
                0,
            ),
            CompactSize {
                prefix: 0,
                number_vec: vec![99],
                number: 99,
            },
            vec![
                64, 53, 168, 238, 71, 229, 33, 142, 142, 201, 164, 167, 44, 212, 25, 184, 30, 38,
                141, 147, 174, 40, 123, 13, 149, 111, 222, 120, 89, 239, 57, 150, 166, 15, 68, 250,
                100, 250, 190, 146, 201, 116, 236, 47, 108, 176, 143, 97, 131, 233, 186, 156, 126,
                183, 137, 133, 130, 115, 136, 242, 43, 175, 197, 37, 15, 33, 3, 218, 43, 97, 162,
                214, 57, 234, 192, 22, 188, 37, 109, 93, 175, 205, 94, 91, 219, 120, 183, 207, 135,
                240, 196, 89, 232, 101, 2, 82, 84, 187, 90,
            ],
            4294967295,
        )],
        CompactSize {
            prefix: 0,
            number_vec: vec![3],
            number: 3,
        },
        vec![Output {
            value: 1084265,
            script_length: CompactSize {
                prefix: 0,
                number_vec: vec![34],
                number: 34,
            },
            script: vec![
                118, 169, 29, 119, 50, 68, 122, 88, 105, 110, 75, 56, 75, 97, 113, 117, 110, 112,
                89, 103, 106, 110, 71, 121, 67, 89, 99, 103, 72, 86, 98, 51, 136, 172,
            ],
        }],
        1687764990,
    )
}
