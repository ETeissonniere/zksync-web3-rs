mod zks_signer_tests {
    use crate::zks_provider::ZKSProvider;
    use crate::zks_utils::ETH_CHAIN_ID;
    use crate::zks_wallet::ZKSWallet;
    use crate::{tests::utils::*, DepositRequest};
    use ethers::abi::Tokenize;
    use ethers::providers::Middleware;
    use ethers::signers::{LocalWallet, Signer};
    use ethers::types::Address;
    use ethers::types::U256;
    use ethers::utils::parse_units;
    use std::fs::File;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_transfer() {
        let receiver_address: Address = "0xa61464658AfeAf65CccaaFD3a512b69A83B77618"
            .parse()
            .unwrap();
        let amount_to_transfer: U256 = 1_i32.into();

        let era_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None).unwrap();

        let sender_balance_before = era_provider
            .get_balance(zk_wallet.l2_address(), None)
            .await
            .unwrap();
        let receiver_balance_before = era_provider
            .get_balance(receiver_address, None)
            .await
            .unwrap();

        println!("Sender balance before: {sender_balance_before}");
        println!("Receiver balance before: {receiver_balance_before}");
        println!("Sender balance before: {sender_balance_before}");
        println!("Receiver balance before: {receiver_balance_before}");

        let receipt = zk_wallet
            .transfer(receiver_address, amount_to_transfer, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        assert_eq!(receipt.from, zk_wallet.l2_address());
        assert_eq!(receipt.to.unwrap(), receiver_address);

        let sender_balance_after = era_provider
            .get_balance(zk_wallet.l2_address(), None)
            .await
            .unwrap();
        let receiver_balance_after = era_provider
            .get_balance(receiver_address, None)
            .await
            .unwrap();

        println!("Sender balance after: {sender_balance_after}");
        println!("Receiver balance after: {receiver_balance_after}");

        assert_eq!(
            sender_balance_after,
            sender_balance_before
                - (amount_to_transfer
                    + receipt.effective_gas_price.unwrap() * receipt.gas_used.unwrap())
        );
        assert_eq!(
            receiver_balance_after,
            receiver_balance_before + amount_to_transfer
        );
    }

    #[tokio::test]
    async fn test_deposit() {
        let l1_provider = eth_provider();
        let l2_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(
            wallet,
            None,
            Some(l2_provider.clone()),
            Some(l1_provider.clone()),
        )
        .unwrap();

        let l1_balance_before = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_before = zk_wallet.era_balance().await.unwrap();
        println!("L1 balance before: {l1_balance_before}");
        println!("L2 balance before: {l2_balance_before}");

        let request = DepositRequest::new(parse_units("0.01", "ether").unwrap().into());
        println!("Amount: {}", request.amount);
        let receipt = zk_wallet.deposit(&request).await.unwrap();
        assert_eq!(receipt.status.unwrap(), 1_u8.into());

        let _l2_receipt = l2_provider
            .get_transaction_receipt(receipt.transaction_hash)
            .await
            .unwrap();

        let l1_balance_after = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_after = zk_wallet.era_balance().await.unwrap();
        println!("L1 balance after: {l1_balance_after}");
        println!("L2 balance after: {l2_balance_after}");

        assert!(
            l1_balance_after <= l1_balance_before - request.amount(),
            "Balance on L1 should be decreased"
        );
        assert!(
            l2_balance_after >= l2_balance_before + request.amount(),
            "Balance on L2 should be increased"
        );
    }

    #[tokio::test]
    async fn test_deposit_to_another_address() {
        let to: Address = "0xa61464658AfeAf65CccaaFD3a512b69A83B77618"
            .parse()
            .unwrap();
        let amount = parse_units("0.01", "ether").unwrap().into();
        println!("Amount: {amount}");

        let request = DepositRequest::new(amount).to(to);

        let l1_provider = eth_provider();
        let l2_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(
            wallet,
            None,
            Some(l2_provider.clone()),
            Some(l1_provider.clone()),
        )
        .unwrap();

        let l1_balance_before = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_before = era_provider().get_balance(to, None).await.unwrap();
        println!("L1 balance before: {l1_balance_before}");
        println!("L2 balance before: {l2_balance_before}");

        let receipt = zk_wallet.deposit(&request).await.unwrap();
        assert_eq!(receipt.status.unwrap(), 1_u8.into());

        let _l2_receipt = l2_provider
            .get_transaction_receipt(receipt.transaction_hash)
            .await
            .unwrap();

        let l1_balance_after = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_after = era_provider().get_balance(to, None).await.unwrap();
        println!("L1 balance after: {l1_balance_after}");
        println!("L2 balance after: {l2_balance_after}");

        assert!(
            l1_balance_after <= l1_balance_before - request.amount(),
            "Balance on L1 should be decreased"
        );
        assert!(
            l2_balance_after >= l2_balance_before + request.amount(),
            "Balance on L2 should be increased"
        );
    }

    #[tokio::test]
    async fn test_transfer_eip712() {
        let receiver_address: Address = "0xa61464658AfeAf65CccaaFD3a512b69A83B77618"
            .parse()
            .unwrap();
        let amount_to_transfer: U256 = 1_i32.into();

        let era_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None).unwrap();

        let sender_balance_before = era_provider
            .get_balance(zk_wallet.l2_address(), None)
            .await
            .unwrap();
        let receiver_balance_before = era_provider
            .get_balance(receiver_address, None)
            .await
            .unwrap();

        println!("Sender balance before: {sender_balance_before}");
        println!("Receiver balance before: {receiver_balance_before}");

        let receipt = zk_wallet
            .transfer_eip712(receiver_address, amount_to_transfer, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        assert_eq!(receipt.from, zk_wallet.l2_address());
        assert_eq!(receipt.to.unwrap(), receiver_address);

        let sender_balance_after = era_provider
            .get_balance(zk_wallet.l2_address(), None)
            .await
            .unwrap();
        let receiver_balance_after = era_provider
            .get_balance(receiver_address, None)
            .await
            .unwrap();

        println!("Sender balance after: {sender_balance_after}");
        println!("Receiver balance after: {receiver_balance_after}");

        assert_eq!(
            sender_balance_after,
            sender_balance_before
                - (amount_to_transfer
                    + receipt.effective_gas_price.unwrap() * receipt.gas_used.unwrap())
        );
        assert_eq!(
            receiver_balance_after,
            receiver_balance_before + amount_to_transfer
        );
    }

    #[tokio::test]
    async fn test_deploy_contract_with_constructor_arg_uint() {
        let era_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None).unwrap();

        let mut contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        contract_path.push("src/abi/test_contracts/storage_combined.json");
        let contract: CompiledContract =
            serde_json::from_reader(File::open(contract_path).unwrap()).unwrap();

        let transaction_receipt = zk_wallet
            .deploy(
                contract.abi,
                contract.bin.to_vec(),
                vec!["10".to_owned()],
                None,
            )
            .await
            .unwrap();

        let contract_address = transaction_receipt.contract_address.unwrap();
        let deploy_result = era_provider.get_code(contract_address, None).await;

        assert!(deploy_result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_contract_with_constructor_arg_string() {
        let era_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None).unwrap();

        let mut contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        contract_path.push("src/abi/test_contracts/greeter_combined.json");
        let contract: CompiledContract =
            serde_json::from_reader(File::open(contract_path).unwrap()).unwrap();

        let transaction_receipt = zk_wallet
            .deploy(
                contract.abi,
                contract.bin.to_vec(),
                vec!["Hey".to_owned()],
                None,
            )
            .await
            .unwrap();

        let contract_address = transaction_receipt.contract_address.unwrap();
        let deploy_result = era_provider.get_code(contract_address, None).await;

        assert!(deploy_result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_contract_with_import() {
        let era_provider = era_provider();
        let wallet = local_wallet();
        let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None).unwrap();

        // Deploy imported contract first.
        let mut contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        contract_path.push("src/abi/test_contracts/counter_combined.json");
        let counter_contract: CompiledContract =
            serde_json::from_reader(File::open(contract_path).unwrap()).unwrap();

        let transaction_receipt = zk_wallet
            .deploy(
                counter_contract.abi,
                counter_contract.bin.to_vec(),
                vec![],
                None,
            )
            .await
            .unwrap();

        let counter_contract_address = transaction_receipt.contract_address.unwrap();
        let deploy_result = era_provider.get_code(counter_contract_address, None).await;

        assert!(deploy_result.is_ok());

        // Deploy another contract that imports the previous one.
        let mut contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        contract_path.push("src/abi/test_contracts/import_combined.json");

        let import_contract: CompiledContract =
            serde_json::from_reader(File::open(contract_path).unwrap()).unwrap();

        let transaction_receipt = zk_wallet
            .deploy(
                import_contract.abi,
                import_contract.bin.to_vec(),
                vec![format!("{counter_contract_address:?}")],
                None,
            )
            .await
            .unwrap();

        let import_contract_address = transaction_receipt.contract_address.unwrap();
        let value = ZKSProvider::call(
            &era_provider,
            import_contract_address,
            "getCounterValue()(uint256)",
            None,
        )
        .await
        .unwrap();

        assert_eq!(value, U256::from(0_u64).into_tokens());
    }

    #[tokio::test]
    async fn test_withdraw_to_same_address() {
        let wallet = local_wallet();
        let zk_wallet =
            ZKSWallet::new(wallet, None, Some(era_provider()), Some(eth_provider())).unwrap();

        // See balances before withdraw
        let l1_balance_before = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_before = zk_wallet.era_balance().await.unwrap();

        println!("Balance on L1 before withdrawal: {l1_balance_before}");
        println!("Balance on L2 before withdrawal: {l2_balance_before}");

        // Withdraw
        let amount_to_withdraw: U256 = parse_units(1_u8, "ether").unwrap().into();
        let tx_receipt = zk_wallet
            .withdraw(amount_to_withdraw, zk_wallet.l1_address())
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        let tx_receipt = zk_wallet
            .get_era_provider()
            .unwrap()
            .wait_for_finalize(tx_receipt, None, None)
            .await
            .unwrap();
        assert_eq!(
            1,
            tx_receipt.status.unwrap().as_u64(),
            "Check that transaction in L2 is successful"
        );

        println!("L2 Transaction hash: {:?}", tx_receipt.transaction_hash);

        let l2_balance_after_withdraw = zk_wallet.era_balance().await.unwrap();
        let l1_balance_after_withdraw = zk_wallet.eth_balance().await.unwrap();

        assert_eq!(
            l2_balance_after_withdraw,
            l2_balance_before
                - (amount_to_withdraw + tx_receipt.effective_gas_price.unwrap() * tx_receipt.gas_used.unwrap()),
            "Check that L2 balance inmediately after withdrawal has decreased by the used gas and amount"
        );

        assert_eq!(
            l1_balance_before, l1_balance_after_withdraw,
            "Check that L1 balance has not changed"
        );

        let tx_finalize_receipt = zk_wallet
            .finalize_withdraw(tx_receipt.transaction_hash)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        println!(
            "L1 Transaction hash: {:?}",
            tx_finalize_receipt.transaction_hash
        );

        assert_eq!(
            1,
            tx_finalize_receipt.status.unwrap().as_u64(),
            "Check that transaction in L1 is successful"
        );

        // See balances after withdraw
        let l1_balance_after_finalize = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_after_finalize = zk_wallet.era_balance().await.unwrap();

        println!("Balance on L1 after finalize withdraw: {l1_balance_after_finalize}");
        println!("Balance on L2 after finalize withdraw: {l2_balance_after_finalize}");

        assert_eq!(
            l2_balance_after_finalize, l2_balance_after_withdraw,
            "Check that L2 balance after finalize has decreased by the used gas"
        );

        assert_ne!(
            l1_balance_after_finalize, l1_balance_before,
            "Check that L1 balance after finalize is not the same"
        );
        assert_eq!(
            l1_balance_after_finalize,
            l1_balance_before
                + (amount_to_withdraw
                    - tx_finalize_receipt.effective_gas_price.unwrap()
                        * tx_finalize_receipt.gas_used.unwrap()),
            "Check that L1 balance after finalize has increased by the amount"
        );
    }

    #[tokio::test]
    async fn test_withdraw_to_other_address() {
        let receiver_private_key =
            "0xe667e57a9b8aaa6709e51ff7d093f1c5b73b63f9987e4ab4aa9a5c699e024ee8";
        let l2_wallet = local_wallet();

        let l1_wallet = LocalWallet::from_str(receiver_private_key)
            .unwrap()
            .with_chain_id(ETH_CHAIN_ID);
        let zk_wallet = ZKSWallet::new(
            l2_wallet,
            Some(l1_wallet),
            Some(era_provider()),
            Some(eth_provider()),
        )
        .unwrap();

        // See balances before withdraw
        let l1_balance_before = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_before = zk_wallet.era_balance().await.unwrap();

        println!("Balance on L1 before withdrawal: {l1_balance_before}");
        println!("Balance on L2 before withdrawal: {l2_balance_before}");

        // Withdraw
        let amount_to_withdraw: U256 = parse_units(1_u8, "ether").unwrap().into();
        let tx_receipt = zk_wallet
            .withdraw(amount_to_withdraw, zk_wallet.l1_address())
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        let tx_receipt = zk_wallet
            .get_era_provider()
            .unwrap()
            .wait_for_finalize(tx_receipt, None, None)
            .await
            .unwrap();
        assert_eq!(
            1,
            tx_receipt.status.unwrap().as_u64(),
            "Check that transaction in L2 is successful"
        );

        println!("L2 Transaction hash: {:?}", tx_receipt.transaction_hash);

        let l2_balance_after_withdraw = zk_wallet.era_balance().await.unwrap();
        let l1_balance_after_withdraw = zk_wallet.eth_balance().await.unwrap();

        assert_eq!(
            l2_balance_after_withdraw,
            l2_balance_before
                - (amount_to_withdraw + tx_receipt.effective_gas_price.unwrap() * tx_receipt.gas_used.unwrap()),
            "Check that L2 balance inmediately after withdrawal has decreased by the used gas and amount"
        );

        assert_eq!(
            l1_balance_before, l1_balance_after_withdraw,
            "Check that L1 balance has not changed"
        );

        let tx_finalize_receipt = zk_wallet
            .finalize_withdraw(tx_receipt.transaction_hash)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        println!(
            "L1 Transaction hash: {:?}",
            tx_finalize_receipt.transaction_hash
        );

        assert_eq!(
            1,
            tx_finalize_receipt.status.unwrap().as_u64(),
            "Check that transaction in L1 is successful"
        );

        // See balances after withdraw
        let l1_balance_after_finalize = zk_wallet.eth_balance().await.unwrap();
        let l2_balance_after_finalize = zk_wallet.era_balance().await.unwrap();

        println!("Balance on L1 after finalize withdraw: {l1_balance_after_finalize}");
        println!("Balance on L2 after finalize withdraw: {l2_balance_after_finalize}");

        assert_eq!(
            l2_balance_after_finalize, l2_balance_after_withdraw,
            "Check that L2 balance after finalize has decreased by the used gas"
        );

        assert_ne!(
            l1_balance_after_finalize, l1_balance_before,
            "Check that L1 balance after finalize is not the same"
        );
        assert_eq!(
            l1_balance_after_finalize,
            l1_balance_before
                + (amount_to_withdraw
                    - tx_finalize_receipt.effective_gas_price.unwrap()
                        * tx_finalize_receipt.gas_used.unwrap()),
            "Check that L1 balance after finalize has increased by the amount"
        );
    }
}