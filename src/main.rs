use std::collections::HashMap;

// fn main() {
//     println!("Hello, Coreum!");
// }

struct MultiSend {
    inputs: Vec<Balance>,
    outputs: Vec<Balance>,
}

pub struct Coin {
    pub denom: String,
    pub amount: i128,
}

struct Balance {
    address: String,
    coins: Vec<Coin>,
}

struct DenomDefinition {
    denom: String,
    issuer: String,
    burn_rate: f64,
    commission_rate: f64,
}

fn calculate_balance_changes(
    original_balances: Vec<Balance>,
    definitions: Vec<DenomDefinition>,
    multi_send_tx: MultiSend,
) -> Result<Vec<Balance>, String> {
    // Check if any balances are empty
    for balance in &original_balances {
        if balance.coins.is_empty() {
            return Err(format!("No balances found for account: {}", balance.address));
        }
    }

    let total_input_amount: i128 = multi_send_tx
        .inputs
        .iter()
        .flat_map(|balance| balance.coins.iter())
        .map(|coin| coin.amount)
        .sum();

    let total_output_amount: i128 = multi_send_tx
        .outputs
        .iter()
        .flat_map(|balance| balance.coins.iter())
        .map(|coin| coin.amount)
        .sum();

    if total_input_amount != total_output_amount {
        return Err("Amounts don't match".to_string());
    }



    let mut issuer_account_found = false;
    //Check if issuer account is present in inputs
    for input in &multi_send_tx.inputs {
        if definitions.iter().any(|d| d.issuer == input.address) {
            issuer_account_found = true;
            break;
        }
    }

    // If issuer account not found in inputs, search in outputs
    if !issuer_account_found {
        for output in &multi_send_tx.outputs {
            if definitions.iter().any(|d| d.issuer == output.address) {
                issuer_account_found = true;
                break;
            }
        }
    }


    let mut updated_balances: HashMap<String, Balance> = HashMap::new();


    // if issuer account is not there in iputs as well as outputs
    if !issuer_account_found {
    // Process inputs
            for input in &multi_send_tx.inputs {
                for coin in &input.coins {
                    let denom = &coin.denom;

                    // Find the corresponding denom definition
                    let denom_definition = definitions.iter().find(|def| def.denom == *denom);

                    if let Some(definition) = denom_definition {
                        let user_updated_amount = (coin.amount as f64) * (1.0 + definition.burn_rate + definition.commission_rate);

                        if definition.commission_rate > 0.0 {
                            let issuer_updated_balance = (coin.amount as f64) * definition.commission_rate;

                            // Update issuer's balance
                            let issuer_balance = updated_balances
                                .entry(definition.issuer.clone())
                                .or_insert_with(|| Balance {
                                    address: definition.issuer.clone(),
                                    coins: vec![],
                                });

                            let issuer_coin = issuer_balance
                                .coins
                                .iter_mut()
                                .find(|coin| &coin.denom == denom);

                            if let Some(issuer_coin) = issuer_coin {
                                issuer_coin.amount += issuer_updated_balance as i128;
                            } else {
                                issuer_balance.coins.push(Coin {
                                    denom: denom.clone(),
                                    amount: issuer_updated_balance as i128,
                                });
                            }
                        }

                        // Update user's balance
                        let user_balance = updated_balances
                            .entry(input.address.clone())
                            .or_insert_with(|| Balance {
                                address: input.address.clone(),
                                coins: vec![],
                            });

                        let user_coin = user_balance.coins.iter_mut().find(|coin| &coin.denom == denom);

                        if let Some(user_coin) = user_coin {
                            user_coin.amount += user_updated_amount as i128;
                        } else {
                            user_balance.coins.push(Coin {
                                denom: denom.clone(),
                                amount: user_updated_amount as i128,
                            });
                        }
                    } else {
                        return Err(format!("No definition found for denom: {}", coin.denom));
                    }
                }
            }

            // Process outputs
            for output in &multi_send_tx.outputs {
                for coin in &output.coins {
                    let denom = &coin.denom;

                    // Find the corresponding denom definition
                    let denom_definition = definitions.iter().find(|def| def.denom == *denom);

                    if let Some(definition) = denom_definition {
                        // Update recipient's balance
                        let recipient_balance = updated_balances
                            .entry(output.address.clone())
                            .or_insert_with(|| Balance {
                                address: output.address.clone(),
                                coins: vec![],
                            });

                        let recipient_coin = recipient_balance
                            .coins
                            .iter_mut()
                            .find(|coin| &coin.denom == denom);

                        if let Some(recipient_coin) = recipient_coin {
                            recipient_coin.amount += coin.amount;
                        } else {
                            recipient_balance.coins.push(Coin {
                                denom: denom.clone(),
                                amount: coin.amount,
                            });
                        }
                    } else {
                        return Err(format!("No definition found for denom: {}", coin.denom));
                    }
                }
            }
    }else{
         // if issuer account is present in put or output

    
        //finding sum of amounts in inputs
            let total_input_amount: i128 = multi_send_tx
            .inputs
            .iter()
            .filter(|input| input.address != "issuer_account_A")
            .flat_map(|input| input.coins.iter())
            .map(|coin| coin.amount)
            .sum();

        //finding sum of amounts in output

            let total_output_amount: i128 = multi_send_tx
            .outputs
            .iter()
            .filter(|output| output.address != "issuer_account_A")
            .flat_map(|output| output.coins.iter())
            .map(|coin| coin.amount)
            .sum();

            //finsing minimum amount
            let minimum_amount = total_input_amount.min(total_output_amount);

            for input in &multi_send_tx.inputs {
                for coin in &input.coins {
                    let denom = &coin.denom;

                    // Find the corresponding denom definition
                    let denom_definition = definitions.iter().find(|def| def.denom == *denom);

                    if let Some(definition) = denom_definition {
                        let user_updated_amount = (((coin.amount as f64) * definition.burn_rate * minimum_amount as f64) / total_input_amount as f64).ceil() as u64;


                        if definition.commission_rate > 0.0 {
                            let issuer_updated_balance = (((coin.amount as f64) * definition.commission_rate * minimum_amount as f64)/total_input_amount as f64).ceil() as u64;

                            // Update issuer's balance
                            let issuer_balance = updated_balances
                                .entry(definition.issuer.clone())
                                .or_insert_with(|| Balance {
                                    address: definition.issuer.clone(),
                                    coins: vec![],
                                });

                            let issuer_coin = issuer_balance
                                .coins
                                .iter_mut()
                                .find(|def| def.denom == *denom);

                            if let Some(issuer_coin) = issuer_coin {
                                issuer_coin.amount += issuer_updated_balance as i128;
                            } else {
                                issuer_balance.coins.push(Coin {
                                    denom: denom.clone(),
                                    amount: issuer_updated_balance as i128,
                                });
                            }
                        }

                        // Update user's balance
                        let user_balance = updated_balances
                            .entry(input.address.clone())
                            .or_insert_with(|| Balance {
                                address: input.address.clone(),
                                coins: vec![],
                            });

                        let user_coin = user_balance.coins.iter_mut().find(|def| def.denom == *denom);

                        if let Some(user_coin) = user_coin {
                            user_coin.amount += user_updated_amount as i128;
                        } else {
                            user_balance.coins.push(Coin {
                                denom: denom.clone(),
                                amount: user_updated_amount as i128,
                            });
                        }
                    } else {
                        return Err(format!("No definition found for denom: {}", coin.denom));
                    }
                }
            }

            // Process outputs
            for output in &multi_send_tx.outputs {
                for coin in &output.coins {
                    let denom = &coin.denom;

                    // Find the corresponding denom definition
                    let denom_definition = definitions.iter().find(|def| def.denom == *denom);

                    if let Some(definition) = denom_definition {
                        // Update recipient's balance
                        let recipient_balance = updated_balances
                            .entry(output.address.clone())
                            .or_insert_with(|| Balance {
                                address: output.address.clone(),
                                coins: vec![],
                            });

                        let recipient_coin = recipient_balance
                            .coins
                            .iter_mut()
                            .find(|def| def.denom == *denom);

                        if let Some(recipient_coin) = recipient_coin {
                            recipient_coin.amount += coin.amount;
                        } else {
                            recipient_balance.coins.push(Coin {
                                denom: denom.clone(),
                                amount: coin.amount,
                            });
                        }
                    } else {
                        return Err(format!("No definition found for denom: {}", coin.denom));
                    }
                }
            }


    }



    let updated_balances: Vec<Balance> = updated_balances.into_iter().map(|(_, v)| v).collect();

    Ok(updated_balances)
}




fn main() {
    // create original_balances
    let original_balances = vec![
        Balance {
            address: "account1".to_string(),
            coins: vec![Coin {
                denom: "denom1".to_string(),
                amount: 1000_000,
            }],
        },
        Balance {
            address: "account2".to_string(),
            coins: vec![Coin {
                denom: "denom2".to_string(),
                amount: 1000_000,
            }],
        },
    ];

    // create definitions
    let definitions = vec![
        DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.08,
            commission_rate: 0.12,
        },
        DenomDefinition {
            denom: "denom2".to_string(),
            issuer: "issuer_account_B".to_string(),
            burn_rate: 1.0,
            commission_rate: 0.0,
        },
    ];

    // create multi_send_tx
    let multi_send_tx = MultiSend {
        inputs: vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin {
                    denom: "denom1".to_string(),
                    amount: 1000,
                }],
            },
            Balance {
                address: "account2".to_string(),
                coins: vec![Coin {
                    denom: "denom2".to_string(),
                    amount: 1000,
                }],
            },
        ],
        outputs: vec![
            Balance {
                address: "account_recipient".to_string(),
                coins: vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 1000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 1000,
                    },
                ],
            },
        ],
    };

    // call calculate_balance_changes
    let result = calculate_balance_changes(original_balances, definitions, multi_send_tx);

    // handle the result
    match result {
        Ok(updated_balances) => {
            // print the updated balances
            for balance in updated_balances {
                println!("Address: {}", balance.address);
                for coin in balance.coins {
                    println!("Denom: {}, Amount: {}", coin.denom, coin.amount);
                }
            }
        }
        Err(err) => {
            // handle the error
            println!("Error: {}", err);
        }
    }
}
