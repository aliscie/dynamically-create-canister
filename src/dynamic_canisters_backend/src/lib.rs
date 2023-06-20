use std::cell::RefCell;

use candid::Principal;
use ic_cdk::{api, caller};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterSettings;
use ic_cdk::export::candid::{
    candid_method, CandidType, check_prog, Deserialize, export_service, IDLProg, TypeEnv,
};
use serde_bytes;


#[derive(Default)]
struct WalletWASMBytes(Option<serde_bytes::ByteBuf>);

/// The wallet (this canister's) name.
#[derive(Default)]
struct WalletName(pub(crate) Option<String>);


thread_local! {
    static WALLET_NAME: RefCell<WalletName> = Default::default();
    static WALLET_WASM_BYTES: RefCell<WalletWASMBytes> = Default::default();
}


#[ic_cdk::query]
#[candid_method(query)]
fn my_canister_id() -> String {
    format!("{}", ic_cdk::id())
}


#[derive(CandidType, Deserialize)]
struct CreateResult {
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct CanisterInstall {
    mode: InstallMode,
    canister_id: Principal,
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
    arg: Vec<u8>,
}


#[derive(CandidType, Deserialize)]
enum InstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade,
}


#[ic_cdk::update]
#[candid_method(update)]
async fn create_canister() -> String {
    #[derive(CandidType)]
    struct In {
        settings: Option<CanisterSettings>,
    }
    let settings = CanisterSettings {
        controllers: Some(vec![caller(), ic_cdk::api::id()]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };
    let in_arg = In {
        settings: Some(settings),
    };

    let (create_result, ): (CreateResult, ) = match api::call::call_with_payment128(
        Principal::management_canister(),
        "create_canister",
        (in_arg, ),
        7_692_307_692,
    )
        .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            );
        }
    };

    // wasm install
    // cargo build --release --target --package dynamic_canisters_backend --locked
    const wasm_module: &[u8] = std::include_bytes!(
        "../dynamic_canisters_backend.wasm"
    );

//     let wasm_module = WALLET_WASM_BYTES.with(|wallet_bytes| match &wallet_bytes.borrow().0 {
//         Some(o) => o.clone().into_vec(),
//         None => {
//             ic_cdk::trap("No wasm module stored.");
//         }
// });

    let install_config = CanisterInstall {
        mode: InstallMode::Install,
        canister_id: ic_cdk::api::id(),
        wasm_module: wasm_module.to_vec().clone(),
        arg: b" ".to_vec(),
    };
    // ic_cdk::print(format!("install_config: {:?}", [Principal::management_canister().to_text()]));

    let (install_res, ): (CreateResult, ) = match api::call::call(
        ic_cdk::api::id(),
        // Principal::management_canister(),
        "install_code",
        (install_config, ),
    ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            );
        }
    };
    // ("An error happened during the call: 3: IC0302: Canister cinef-v4aaa-aaaaa-qaalq-cai has no update method 'install_code'")
    format!("{}", create_result.canister_id)
}


#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::env;
    use std::fs::{create_dir_all, write};
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;

    use candid::export_service;
    use ic_cdk::{api, update};
    use ic_cdk::api::management_canister::main::CanisterSettings;
    // use ic_cdk::export::candid::{
    //     candid_method, CandidType, check_prog, Deserialize, export_service, IDLProg, TypeEnv,
    // };
    use ic_cdk::export::candid::Principal;

    #[test]
    fn save_candid_2() {
        #[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
        fn export_candid() -> String {
            ic_cdk::export::candid::export_service!();
            __export_service()
        }

        let mut dir: PathBuf = env::current_dir().unwrap();
        let canister_name: Cow<str> = dir.file_name().unwrap().to_string_lossy();

        match create_dir_all(&dir) {
            Ok(_) => println!("Successfully created directory"),
            Err(e) => println!("Failed to create directory: {}", e),
        }

        let res = write(dir.join(format!("{:?}.did", canister_name).replace("\"", "")), export_candid());
        println!("-------- Wrote to {:?}", dir);
        println!("-------- res {:?}", canister_name);
    }
}