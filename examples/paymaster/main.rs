//! The below example demonstrates how to craft a transaction to call a contract method and get
//! fees paid for by a Paymaster contract.

use ethers::abi::HumanReadableParser;
use std::str::FromStr;
use zksync_web3_rs::{
    eip712::{Eip712Meta, Eip712TransactionRequest, PaymasterParams},
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    zks_provider::ZKSProvider,
    zks_utils,
    zks_wallet::CallRequest,
    ZKSWallet,
};

mod helpers;

// We use a local dockerized node for this example
static ERA_PROVIDER_URL: &str = "http://127.0.0.1:3050";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (greeter_address, paymaster_address) =
        helpers::setup_environment(ERA_PROVIDER_URL).await.unwrap();

    // Make a new wallet, with no funds to pay for gas fees
    let zk_wallet = {
        let era_provider = Provider::try_from(ERA_PROVIDER_URL).unwrap();
        let chain_id = era_provider.get_chainid().await.unwrap();
        let l2_wallet = LocalWallet::from_str(
            // this was generated randomly via https://iancoleman.io/bip39/#english
            "d06b37324fabff929a36d1af926b05e221647e0a2966b4fc2be3e1fa5177fbe9",
        )
        .unwrap()
        .with_chain_id(chain_id.as_u64());

        println!("Wallet address: {:?}", l2_wallet.address());
        println!(
            "Current wallet balance (expected to be 0): {:?}",
            era_provider
                .get_balance(l2_wallet.address(), None)
                .await
                .unwrap()
        );

        ZKSWallet::new(l2_wallet, None, Some(era_provider.clone()), None).unwrap()
    };

    // Change the greeting, using the paymaster to pay for our gas fees
    {
        // Paymaster arguments need to be specified via a `Eip712TransactionRequest`,
        // though it isn't necessary to specify ALL the fields in there as
        // `send_transaction_eip712` will set most of them for us.

        // 1. Compose tx data for the Greeter contract
        let function = HumanReadableParser::parse_function("setGreeting(string)").unwrap();
        let function_args = function
            .decode_input(&zks_utils::encode_args(&function, &["Hello Paymaster!"]).unwrap())
            .unwrap();

        // 2. Compose tx data for the Paymaster
        let meta = Eip712Meta::new().paymaster_params(
            PaymasterParams::default().paymaster(paymaster_address), // .paymaster_input(input.into()),
        );

        // 3. Put it all together
        let tx_request = Eip712TransactionRequest::new()
            .to(greeter_address)
            .data(function.encode_input(&function_args).unwrap())
            .custom_data(meta);

        // 4. Send it off
        let era_provider = zk_wallet.get_era_provider().unwrap();
        let pending = era_provider
            .send_transaction_eip712(&zk_wallet.l2_wallet, tx_request)
            .await
            .unwrap();
        let receipt = pending.await.unwrap().unwrap();

        println!("Transaction sent: {:?}", receipt.transaction_hash);
    }

    // Call the greet view method to demonstrate our call did go through
    {
        let era_provider = zk_wallet.get_era_provider().unwrap();
        let call_request = CallRequest::new(greeter_address, "greet()(string)".to_owned());

        let greet = ZKSProvider::call(era_provider.as_ref(), &call_request)
            .await
            .unwrap();

        println!("Greeting value: {}", greet[0]);
    }
}
