use crate::layout::rt::cvlr_new_account_info;
use rstest::rstest;
use crate::layout::rt::instruction_accounts::{
    AccountData, AccountMeta, InstructionAccounts, InstructionAccountsBuilder,
};
use solana_program::pubkey::Pubkey;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

fn cache_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = Path::new(&manifest_dir).join(".cache").join("rpc");
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fetch_with_backoff(body: serde_json::Value) -> serde_json::Value {
    let mut delay = std::time::Duration::from_millis(1000);
    let mut next_delay = || {
        let current_delay = delay;
        delay *= 2;
        current_delay
    };

    let max_retries = 12;

    let url = "https://api.mainnet-beta.solana.com";

    for attempt in 0..max_retries {
        match ureq::post(url).send_json(&body) {
            Ok(response) => return response.into_json().unwrap(),
            Err(ureq::Error::Status(429, _)) | Err(ureq::Error::Status(503, _)) => {
                std::thread::sleep(next_delay());
            }

            Err(e) => panic!("rpc error: {e}"),
        }
    }

    panic!("rpc failed after {max_retries} retries");
}

fn as_usize(value: &serde_json::Value) -> usize {
    value.as_u64().unwrap().try_into().unwrap()
}

fn as_array_iter(value: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> + Clone {
    value.as_array().into_iter().flatten()
}

fn into_pubkey(value: &serde_json::Value) -> Pubkey {
    let value = value.as_str().unwrap();
    Pubkey::from_str(value).unwrap()
}

fn decode_base64(value: &serde_json::Value) -> Vec<u8> {
    use base64::engine::general_purpose;
    use base64::Engine as _;

    let value = value.as_str().unwrap();
    general_purpose::STANDARD.decode(value).unwrap()
}

fn fetch_tx_accounts(sig: &str) -> Vec<AccountMeta> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [sig, {"encoding": "json", "maxSupportedTransactionVersion": 0}],
    });
    let response = fetch_with_backoff(body);

    let meta = &response["result"]["meta"];
    let header = &response["result"]["transaction"]["message"]["header"];

    let account_keys = as_array_iter(&response["result"]["transaction"]["message"]["accountKeys"]);

    let num_required_signatures = as_usize(&header["numRequiredSignatures"]);
    let num_readonly_signed = as_usize(&header["numReadonlySignedAccounts"]);
    let num_readonly_unsigned = as_usize(&header["numReadonlyUnsignedAccounts"]);
    let num_keys = account_keys.clone().count();
    let num_writable_signed = num_required_signatures - num_readonly_signed;
    let num_writable_unsigned = num_keys - num_required_signatures - num_readonly_unsigned;

    let accounts = account_keys.enumerate().map(|(i, value)| {
        let key = into_pubkey(value);
        let is_signer = i < num_required_signatures;
        let is_writable = if is_signer {
            i < num_writable_signed
        } else {
            i - num_required_signatures < num_writable_unsigned
        };

        AccountMeta {
            key,
            is_signer,
            is_writable,
        }
    });

    let loaded_addresses = &meta["loadedAddresses"];
    let writable = as_array_iter(&loaded_addresses["writable"]).map(|value| AccountMeta {
        key: into_pubkey(value),
        is_signer: false,
        is_writable: true,
    });
    let read_only = as_array_iter(&loaded_addresses["readonly"]).map(|value| AccountMeta {
        key: into_pubkey(value),
        is_signer: false,
        is_writable: false,
    });

    accounts.chain(writable).chain(read_only).collect()
}

fn fetch_account(meta: AccountMeta) -> Option<AccountData> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [meta.key, {"encoding": "base64"}],
    });
    let response = fetch_with_backoff(body);

    let value = &response["result"]["value"];
    if value.is_null() {
        None
    } else {
        let account = AccountData {
            meta,
            lamports: value["lamports"].as_u64().unwrap(),
            data: decode_base64(&value["data"]),
            owner: Pubkey::from_str(value["owner"].as_str().unwrap()).unwrap(),
            executable: value["executable"].as_bool().unwrap(),
            rent_epoch: value["rentEpoch"].as_u64().unwrap_or(u64::MAX),
        };

        Some(account)
    }
}

fn test_signature(signature: &str) {
    let fetched: Vec<AccountData> = fetch_tx_accounts(signature)
        .into_iter()
        .filter_map(fetch_account)
        .collect();

    let mut bytes = Vec::new();
    for account in &fetched {
        account.serialize(&mut bytes);
    }

    let builder = InstructionAccountsBuilder::from_bytes(&bytes);
    InstructionAccounts::init_from_builder(builder);

    for expected in &fetched {
        let account_info = cvlr_new_account_info();

        assert_eq!(*account_info.key, expected.meta.key);
        assert_eq!(*account_info.owner, expected.owner);
        assert_eq!(account_info.lamports(), expected.lamports);
        assert_eq!(&**account_info.data.borrow(), expected.data.as_slice());
        assert_eq!(account_info.is_signer, expected.meta.is_signer);
        assert_eq!(account_info.is_writable, expected.meta.is_writable);
        assert_eq!(account_info.executable, expected.executable);
        assert_eq!(account_info.rent_epoch, expected.rent_epoch);
    }

    println!("\nall assertions passed");
}

#[rstest]
#[case("5ufaoETMik6fsq9JwK4AAXRp4FkBK441piYDBJdUs2yEiQsMZ9sdGu6SZdzqsGSYUnBEY84wtNyKCXWQXfdWmKkE")]
#[case("PP4JFV6mAgMXyPNKiwGDfn32cAK9PhbmZq3Q3a4zLrTgpcr2PSHoMuYpZorDbCbAbGMXs5TU3E22cvYyWM6pnrv")]
#[case("zteAufXF81HoNu7UzBC9GkFPdxXLkEywzRoKg6XpbBk72vJsPvHkUTJTnxV2hBkn9AE5e8VpFUVd6CBefqpDeP7")]
#[case("2CwfxCYs4gi7FQWH1Z4VqSPD4J1xidMxS5M3wPLoyTcmfcygCLLEXu4NMfcXToHjJ6mLW772Kro6ifuyeoCTPCGc")]
#[case("5RYkMUt27XhwgbcB6p4QNP5JKRf354ZAURhBeiHcYBa5LP4qA7jEFdCF3AGHnpxogMdaLSjJFfdfwjURgJrPfmAT")]
#[case("3YGnAQz8uuoFee3qqr7Q9NwDRGPRmorZzLydcszkVYQW58GgBbDkemmaPApQxWDzawCZhCMeJZmL36Fn24xZE8Bd")]
#[case("iFPJR4TqhadRgQbtJu6wEntL4W8P51o4Huxm5mFW2g2ujntSgjnCCqWsUrnVyc1WvP3ekxGE4t5BqSY1iuJsdV6")]
#[case("mm26W7aYuqZWtjsxojwG4x8o6bBULZPpxjPcW8QzfgUKHfwFnrtQx7VRvGnJGEjp5APG6s4nhwqw1PDL8eAPsGw")]
#[case("29tqtUwwiB3XkZP52euYYSVpwSUqmy5o4QDUddeSiti3TTjw93YjvUzVKPTqFHExEoLLHk8fkxjH16t41ETUJNQ3")]
#[case("3WsdcKN1UACHi4kRNDWgDbptf8DCqPN1eVXSCU55Ft5u5VXU4BAaonU8Yc2fewPU3MXEpryoM8GgT27BPoL4rcNX")]
#[case("5pNw5NSnx9qSErCxJ88b7dJA4VKZkMQz7U3GC6E335rjxGyuciLB1dJmCp47xSZC9C8LxvoSgDA2SeUM1io5Hdhd")]
#[case("3Xi9tWhJ6gZbPXqQuCpfcwUak4KRggm21MD88m4qnirU5dVt5G4fM3ksD8uR5jCkmAe17x7QaRxB4h2Po9VEDMY4")]
#[case("5P7vWBBYgSWnbXRHsKSSZFkV9eDnXbq2yKLZAwpHACBC6TEwdeNNfE6WGwBgV56HPEva4i78DGyNnabBjBjYGa71")]
fn deserialize(#[case] sig: &str) {
    test_signature(sig);
}
