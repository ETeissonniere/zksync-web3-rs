use ethers::{
    abi::Abi,
    core::k256::{
        ecdsa::{RecoveryId, Signature as RecoverableSignature},
        schnorr::signature::hazmat::PrehashSigner,
    },
    types::H160,
};
use std::str::FromStr;
use zksync_web3_rs::{
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    zks_provider::ZKSProvider,
    zks_wallet::{DeployRequest, TransferRequest},
    ZKSWallet, ZKSWalletError,
};

// This is the private key for one of the rich wallets that come bundled with the era-test-node.
// It is only used to deploy the contracts necessary to this example.
static PRIVATE_KEY: &str = "7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

pub static GREETER_BIN: &str = include_str!("./Greeter.bin");
pub static GREETER_ABI: &str = include_str!("./Greeter.abi");

pub static PAYMASTER_BIN: &str = include_str!("./Paymaster.bin");
pub static PAYMASTER_ABI: &str = include_str!("./Paymaster.abi");

pub async fn deploy_greeter<M, D>(wallet: ZKSWallet<M, D>) -> Result<H160, ZKSWalletError<M, D>>
where
    M: Middleware + 'static + Clone + ZKSProvider,
    D: PrehashSigner<(RecoverableSignature, RecoveryId)> + Sync + Send + Clone,
{
    let abi = Abi::load(GREETER_ABI.as_bytes()).unwrap();
    let contract_bin = hex::decode(GREETER_BIN).unwrap().to_vec();
    let request =
        DeployRequest::with(abi, contract_bin, vec!["Hey".to_owned()]).from(wallet.l2_address());
    let address = wallet.deploy(&request).await?;

    println!("Greeter address: {:#?}", address);

    Ok(address)
}

pub async fn deploy_paymaster<M, D>(wallet: ZKSWallet<M, D>) -> Result<H160, ZKSWalletError<M, D>>
where
    M: Middleware + 'static + Clone + ZKSProvider,
    D: PrehashSigner<(RecoverableSignature, RecoveryId)> + Sync + Send + Clone,
{
    let abi = Abi::load(PAYMASTER_ABI.as_bytes()).unwrap();
    let contract_bin = hex::decode(PAYMASTER_BIN).unwrap().to_vec();
    let request = DeployRequest::with(abi, contract_bin, vec![]).from(wallet.l2_address());
    let address = wallet.deploy(&request).await?;

    println!("Paymaster address: {:#?}", address);

    Ok(address)
}

pub async fn setup_environment(
    era_provider_url: &str,
) -> Result<(H160, H160), Box<dyn std::error::Error>> {
    // Init wallet used for deployment and initial funding
    let zk_wallet = {
        let era_provider = Provider::try_from(era_provider_url).unwrap();
        let chain_id = era_provider.get_chainid().await.unwrap();
        let l2_wallet = LocalWallet::from_str(PRIVATE_KEY)
            .unwrap()
            .with_chain_id(chain_id.as_u64());
        ZKSWallet::new(l2_wallet, None, Some(era_provider.clone()), None).unwrap()
    };

    // Deploy contracts
    let greeter_address = deploy_greeter(zk_wallet.clone()).await?;
    let paymaster_address = deploy_paymaster(zk_wallet.clone()).await?;

    // Fund paymaster
    let tx_id = zk_wallet
        .transfer(
            // 1 ETH
            &TransferRequest::new(1_000_000_000_000_000_000u64.into())
                .to(paymaster_address)
                .from(zk_wallet.l2_address()),
            None,
        )
        .await?;

    println!("Paymaster funded via transaction: {:#?}", tx_id);
    println!(
        "Paymaster balance: {:?}",
        zk_wallet
            .get_era_provider()
            .unwrap()
            .get_balance(paymaster_address, None)
            .await
            .unwrap(),
    );

    Ok((greeter_address, paymaster_address))
}
