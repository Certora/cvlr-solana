use crate::layout::common::sizes;
use core::iter::repeat_with;
use solana_program::entrypoint;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;

/// internal representation of account data,
/// used only during build phase and then serialized
/// to bytes as per Solana ABI.
#[derive(Debug, Default)]
pub(crate) struct AccountData {
    pub meta: AccountMeta,
    pub executable: bool,
    pub owner: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub rent_epoch: u64,
}

#[derive(Debug, Default)]
pub(crate) struct AccountMeta {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}


impl AccountData {
    pub fn parse(mut bytes: &[u8]) -> Vec<AccountData> {
        core::iter::from_fn(|| {
            let account;
            (account, bytes) = AccountData::parse_next(bytes)?;
            Some(account)
        })
        .collect()
    }

    fn parse_next(mut bytes: &[u8]) -> Option<(AccountData, &[u8])> {
        let start = bytes.len();
        match *bytes.first()? {
            entrypoint::NON_DUP_MARKER => {
                bytes = &bytes[sizes::NON_DUP_MARKER..];
            }
            duped_from => {
                // duplicate account detected:
                // the byte is the index of the account that has appeared prior.
                // (might want to do something else here instead)
                panic!("detected dupe of account with index {duped_from}");
            }
        }

        let is_signer = *bytes.first()? != 0;
        bytes = &bytes[sizes::IS_SIGNER..];

        let is_writable = *bytes.first()? != 0;
        bytes = &bytes[sizes::IS_WRITABLE..];

        let executable = *bytes.first()? != 0;
        bytes = &bytes[sizes::EXECUTABLE..];

        bytes = &bytes[sizes::ORIGINAL_DATA_LEN..];

        let key = {
            let chunk = bytes.first_chunk()?;
            Pubkey::from(*chunk)
        };
        bytes = &bytes[sizes::KEY..];

        let owner = {
            let chunk = bytes.first_chunk()?;
            Pubkey::from(*chunk)
        };
        bytes = &bytes[sizes::OWNER..];

        let lamports = {
            let chunk = bytes.first_chunk()?;
            u64::from_le_bytes(*chunk)
        };
        bytes = &bytes[sizes::LAMPORTS..];

        let data = {
            let chunk = bytes.first_chunk()?;
            let data_len = usize::from_le_bytes(*chunk);
            bytes = &bytes[sizes::DATA_LEN_FIELD..];
            let data = bytes.get(..data_len)?.to_owned();
            bytes = &bytes[data_len..];
            data
        };

        bytes = &bytes[sizes::MAX_PERMITTED_DATA_INCREASE..];

        // note the order of operands here, the array shrinks as we go further
        let offset_from_start = start - bytes.len();
        bytes = &bytes[sizes::padding(offset_from_start)..];

        let rent_epoch = {
            let chunk = bytes.first_chunk()?;
            u64::from_le_bytes(*chunk)
        };
        bytes = &bytes[sizes::RENT_EPOCH..];
        let meta = AccountMeta {
            key,
            is_signer,
            is_writable,
        };
        let account = AccountData {
            meta,
            executable,
            owner,
            lamports,
            data,
            rent_epoch,
        };

        Some((account, bytes))
    }

    pub fn max_len(&self) -> usize {
        /// the "worst-case" amount of padding possible,
        /// which we use to ensure capacity
        let max_padding = sizes::BPF_ALIGN_OF_U128 - 1;

        let max_len_in_build_phase = sizes::NON_DUP_MARKER
            + sizes::IS_SIGNER
            + sizes::IS_WRITABLE
            + sizes::EXECUTABLE
            + sizes::ORIGINAL_DATA_LEN
            + sizes::KEY
            + sizes::OWNER
            + sizes::LAMPORTS
            + sizes::DATA_LEN_FIELD
            + self.data.len() // note that element count = data_len, because size_of u8 = 1
            + sizes::MAX_PERMITTED_DATA_INCREASE
            + max_padding
            + sizes::RENT_EPOCH;

        // we also allow data growth after build phase.
        max_len_in_build_phase + sizes::MAX_PERMITTED_DATA_INCREASE
    }

    /// serialize into `buf` using the Solana account ABI layout
    ///
    /// note that the account is always serialized
    /// as non-dup.
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        let start = buf.len();

        let current_data_len = self.data.len();
        let original_data_len = u32::try_from(current_data_len).expect("data len fits in u32");

        buf.push(entrypoint::NON_DUP_MARKER);
        buf.push(self.meta.is_signer.into());
        buf.push(self.meta.is_writable.into());
        buf.push(self.executable.into());
        buf.extend_from_slice(&original_data_len.to_le_bytes());
        buf.extend_from_slice(self.meta.key.as_ref());
        buf.extend_from_slice(self.owner.as_ref());
        buf.extend_from_slice(&self.lamports.to_le_bytes());
        buf.extend_from_slice(&current_data_len.to_le_bytes());
        buf.extend_from_slice(&self.data);

        let padding = sizes::padding(buf.len() + sizes::MAX_PERMITTED_DATA_INCREASE - start);
        buf.resize(buf.len() + sizes::MAX_PERMITTED_DATA_INCREASE + padding, 0);

        buf.extend_from_slice(&self.rent_epoch.to_le_bytes());
    }
}

/// setup phase: parses account data from user input and allows configuring
/// individual accounts before they are serialized into [`InstructionAccounts`].
pub struct InstructionAccountsBuilder {
    accounts: Vec<AccountData>,
    /// the number of accounts to allocate, defaults to all of them
    remaining: usize,
}

impl InstructionAccountsBuilder {
    /// initialize the builder by parsing an array of bytes
    /// conforming to the Solana ABI
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let accounts = AccountData::parse(bytes);
        let remaining = accounts.len();

        InstructionAccountsBuilder {
            accounts,
            remaining,
        }
    }

    /// initialize the builder with zeroed account data,
    /// where each account is considered unique (not a duplicate),
    /// and key/owner are set to zero.
    pub fn with_zeroed(count: usize) -> Self {
        let accounts: Vec<_> = repeat_with(AccountData::default).take(count).collect();
        let remaining = accounts.len();

        InstructionAccountsBuilder {
            accounts,
            remaining,
        }
    }

    /// initialize the builder with zeroed account data,
    /// where each account is considered unique (not a duplicate),
    /// and key/owner are unique.
    pub fn with_zeroed_and_unique_pubkeys(count: usize) -> Self {
        let accounts: Vec<_> = repeat_with(|| AccountData {
            meta: AccountMeta {
                key: Pubkey::new_unique(),
                ..Default::default()
            },
            owner: Pubkey::new_unique(),
            ..Default::default()
        })
        .take(count)
        .collect();
        let remaining = accounts.len();

        InstructionAccountsBuilder {
            accounts,
            remaining,
        }
    }

    fn cursor(&self) -> usize {
        self.accounts.len() - self.remaining
    }

    /// finish constructing current account and start constructing next
    /// returns true if, after the cursor has been advanced, there are still
    /// any accounts left
    pub fn next(&mut self) -> bool {
        self.remaining = self.remaining.saturating_sub(1);
        self.remaining > 0
    }

    pub fn set_len(&mut self, new_len: usize) {
        let len = self.accounts.len();

        if new_len > len {
            panic!("requested {new_len} accounts but only {len} were parsed from the input");
        } else if new_len > self.remaining {
            panic!("cannot set len to {new_len} since that would overwrite already-built accounts");
        } else {
            self.remaining = new_len
        }
    }

    pub fn set_signer(&mut self, new_signer: bool) {
        self.current_account_mut().meta.is_signer = new_signer;
    }

    pub fn set_writable(&mut self, new_writable: bool) {
        self.current_account_mut().meta.is_writable = new_writable;
    }

    pub fn set_executable(&mut self, new_executable: bool) {
        self.current_account_mut().executable = new_executable;
    }

    pub fn set_lamports(&mut self, new_lamports: u64) {
        self.current_account_mut().lamports = new_lamports;
    }

    pub fn set_key(&mut self, new_key: &Pubkey) {
        self.current_account_mut().meta.key = *new_key;
    }

    pub fn set_owner(&mut self, new_owner: &Pubkey) {
        self.current_account_mut().owner = *new_owner;
    }

    pub fn set_data(&mut self, new_data: &[u8]) {
        let account = self.current_account_mut();

        if new_data.len() > account.data.len() + sizes::MAX_PERMITTED_DATA_INCREASE {
            panic!("new data len exceeds maximum permitted size increase");
        } else {
            account.data.clear();
            account.data.extend_from_slice(new_data);
        }
    }

    fn current_account_mut(&mut self) -> &mut AccountData {
        let cursor = self.cursor();
        self.accounts
            .get_mut(cursor)
            .unwrap_or_else(|| panic!("cursor is past the end of accounts"))
    }
}

#[derive(Debug)]
pub struct InstructionAccounts {
    /// contiguous backing buffer for all serialized accounts.
    /// pre-allocated to never reallocate, keeping raw pointers stable.
    buf: Vec<u8>,
    /// start offset within `buf` for each account
    start_offsets: Vec<usize>,
    allocated: usize,
}

thread_local! {
    static GLOBAL: RefCell<Option<InstructionAccounts>> = const { RefCell::new(None) };
}

/// convenience function to reduce boilerplate, probably overkill
fn with_global_borrow<R, F: FnOnce(&InstructionAccounts) -> R>(f: F) -> R {
    GLOBAL.with_borrow(|global| {
        let global = global.as_ref().expect("global is initialized");
        f(global)
    })
}

/// convenience function to reduce boilerplate, probably overkill
fn with_global_borrow_mut<R, F: FnOnce(&mut InstructionAccounts) -> R>(f: F) -> R {
    GLOBAL.with_borrow_mut(|global| {
        let global = global.as_mut().expect("global is initialized");
        f(global)
    })
}

impl InstructionAccounts {
    pub fn init_from_builder(builder: InstructionAccountsBuilder) {
        let buf_capacity = builder.accounts.iter().map(AccountData::max_len).sum();
        let mut buf = Vec::with_capacity(buf_capacity);

        let mut start_offsets = Vec::with_capacity(builder.accounts.len());

        for account in builder.accounts {
            start_offsets.push(buf.len());
            account.serialize(&mut buf);
        }

        let accounts = InstructionAccounts {
            buf,
            start_offsets,
            allocated: 0,
        };

        GLOBAL.with_borrow_mut(|global| {
            let old = global.replace(accounts);
            assert!(old.is_none() || cfg!(test), "can only be initialized once");
        });
    }

    pub fn allocated() -> usize {
        with_global_borrow(|accounts| accounts.allocated)
    }

    fn next_ptr_impl(&mut self) -> Option<*mut u8> {
        let offset = *self.start_offsets.get(self.allocated)?;
        self.allocated += 1;

        // SAFETY: `offset` was recorded by `Account::serialize` and is within `buf`.
        // `buf` was pre-allocated with enough capacity to never reallocate,
        // so this pointer remains valid for the lifetime of `self`.
        let ptr = unsafe { self.buf.as_mut_ptr().add(offset) };

        Some(ptr)
    }

    pub(crate) fn next_ptr() -> Option<*mut u8> {
        with_global_borrow_mut(InstructionAccounts::next_ptr_impl)
    }
}
