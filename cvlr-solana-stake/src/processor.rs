//! Instructions
//!
//! Models of the Stake program's instructions; see the [crate] documentation
//! for what the model leaves out.

use std::io::Write;

use cvlr_asserts::cvlr_assume;
use cvlr_nondet::nondet;
use solana_account_info::{next_account_info, AccountInfo};
use solana_clock::Clock;
use solana_program_error::{ProgramError, ProgramResult};
use solana_pubkey::Pubkey;
use solana_stake_interface::{
    stake_flags::StakeFlags,
    state::{Delegation, Lockup, Meta, Stake, StakeAuthorize, StakeStateV2},
};
use solana_sysvar::SysvarSerialize;

use crate::state::{
    empty_stake_flags, nondet_stake, stake_from_account_info_unchecked, stake_to_slice_unchecked,
};

/// Model of `Withdraw`: moves `withdraw_lamports` from a stake account.
///
/// Accounts, in order: stake account, recipient, `Clock` sysvar, stake
/// history (unused), withdraw authority (unused). The reserve that must stay
/// in the stake account is:
///
/// * for `Stake`: the rent-exempt reserve plus the delegated stake, or, once
///   the clock's epoch has reached the deactivation epoch, plus an arbitrary
///   amount no greater than the delegated stake (the stake still cooling
///   down);
/// * for `Initialized`: the rent-exempt reserve;
/// * for `Uninitialized`: zero. Any other state panics.
///
/// Withdrawing the entire balance panics if any stake is still active, and
/// otherwise marks the account `Uninitialized` (its data must be exactly
/// `StakeStateV2::size_of()` bytes). A partial withdrawal panics if it would
/// leave less than the reserve. The lamports are then moved with
/// [`relocate_lamports`]. Lockups and the withdraw authority are not
/// checked.
#[cvlr_early_panic::early_panic]
// The model keeps the stake program's former rent rule, the reserve recorded in
// `Meta`, rather than the current one, `Rent` read at each instruction.
#[allow(deprecated)]
pub fn process_withdraw(accounts: &[AccountInfo], withdraw_lamports: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 5 accounts (2 sysvars)
    let source_stake_account_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let _stake_history_info = next_account_info(account_info_iter)?;
    let _withdraw_authority_info = next_account_info(account_info_iter)?;

    // other accounts
    //let _option_lockup_authority_info = next_account_info(account_info_iter).ok();

    let clock = &Clock::from_account_info(clock_info)?;
    // let stake_history = &StakeHistorySysvar(clock.epoch);

    let (_lockup, reserve, is_staked) = match get_stake_state(source_stake_account_info)? {
        StakeStateV2::Stake(meta, stake, _stake_flag) => {
            // if we have a deactivation epoch and we're in cooldown
            let staked = if clock.epoch >= stake.delegation.deactivation_epoch {
                get_effective_stake(&stake)
                // stake.delegation.stake(
                //     clock.epoch,
                //     stake_history,
                //     crate::PERPETUAL_NEW_WARMUP_COOLDOWN_RATE_EPOCH,
                // )
            } else {
                // Assume full stake if the stake account hasn't been
                //  de-activated, because in the future the exposed stake
                //  might be higher than stake.stake() due to warmup
                stake.delegation.stake
            };

            let staked_and_reserve = staked.checked_add(meta.rent_exempt_reserve).unwrap();
            (meta.lockup, staked_and_reserve, staked != 0)
        }
        StakeStateV2::Initialized(meta) => {
            // stake accounts must have a balance >= rent_exempt_reserve
            (meta.lockup, meta.rent_exempt_reserve, false)
        }
        StakeStateV2::Uninitialized => {
            (Lockup::default(), 0, false) // no lockup, no restrictions
        }
        _ => panic!(),
    };

    // verify that lockup has expired or that the withdrawal is signed by the
    // custodian both epoch and unix_timestamp must have passed
    // if _lockup.is_in_force(clock, custodian) {
    // panic!();
    // }

    let stake_account_lamports = source_stake_account_info.lamports();
    if withdraw_lamports == stake_account_lamports {
        // if the stake is active, we mustn't allow the account to go away
        if is_staked {
            panic!();
        }

        // De-initialize state upon zero balance
        // set_stake_state(source_stake_account_info, &StakeStateV2::Uninitialized)?;
        write_uninitialized_stake(source_stake_account_info)?;
    } else {
        // a partial withdrawal must not deplete the reserve
        let withdraw_lamports_and_reserve = withdraw_lamports.checked_add(reserve).unwrap();
        if withdraw_lamports_and_reserve > stake_account_lamports {
            panic!();
        }
    }

    relocate_lamports(
        source_stake_account_info,
        destination_info,
        withdraw_lamports,
    );

    Ok(())
}

/// Model of `Deactivate`: starts deactivating a delegated stake.
///
/// Accounts, in order: stake account, `Clock` sysvar. The stake account must
/// be in the `Stake` state and not already deactivating (otherwise the model
/// panics); its deactivation epoch is set to the clock's epoch. The stake
/// authority is not checked.
#[cvlr_early_panic::early_panic]
pub fn process_deactivate(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 2 accounts (1 sysvar)
    let stake_account_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    // other accounts
    // let _stake_authority_info = next_account_info(account_info_iter);

    let clock = &Clock::from_account_info(clock_info)?;

    match get_stake_state(stake_account_info)? {
        StakeStateV2::Stake(meta, mut stake, stake_flags) => {
            stake.deactivate(clock.epoch)?;

            set_stake_state(
                stake_account_info,
                &StakeStateV2::Stake(meta, stake, stake_flags),
            )
        }
        _ => panic!(),
    }?;

    Ok(())
}

/// Model of `Authorize`: replaces the staker or withdrawer authority.
///
/// Accounts, in order: stake account, `Clock` sysvar, current authority
/// (unused). For an `Initialized` or `Stake` account, sets
/// `meta.authorized.staker` or `meta.authorized.withdrawer` (by
/// `authority_type`) to `new_authority`; any other state panics. No signer,
/// current-authority or lockup check is made: the update always happens.
#[cvlr_early_panic::early_panic]
pub fn process_authorize(
    accounts: &[AccountInfo],
    new_authority: &Pubkey,
    authority_type: StakeAuthorize,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 3 accounts (1 sysvar)
    let stake_account_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let _stake_or_withdraw_authority_info = next_account_info(account_info_iter)?;

    // other accounts
    // let option_lockup_authority_info = next_account_info(account_info_iter).ok();

    let clock = &Clock::from_account_info(clock_info)?;

    // let custodian = option_lockup_authority_info
    // .filter(|a| a.is_signer)
    // .map(|a| a.key);

    // skips authorization checks and always updates the authority
    do_authorize(
        stake_account_info,
        new_authority,
        authority_type,
        None,
        clock,
    )?;

    Ok(())
}

/// Model of `DelegateStake`: delegates a stake account to a vote account.
///
/// Accounts, in order: stake account, vote account, `Clock` sysvar, stake
/// history (unused), stake config (unused). Only the vote account's key is
/// used; it is not checked to be a vote account. The amount delegated is the
/// account's lamports minus its rent-exempt reserve, and the model panics if
/// that is below the minimum delegation.
///
/// * `Initialized`: the account becomes `Stake`, delegated to the vote
///   account from the clock's epoch, with arbitrary observed credits and no
///   flags.
/// * `Stake`: if the effective stake (an arbitrary amount no greater than the
///   delegated stake) is non-zero, the model panics unless the voter is the
///   same and the stake was deactivated in the current epoch, in which case
///   the deactivation is cancelled. Otherwise the delegation is reset to the
///   new amount, voter and epoch, with arbitrary observed credits.
/// * Any other state panics.
///
/// The stake authority is not checked.
#[cvlr_early_panic::early_panic]
pub fn process_delegate(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 5 accounts (2 sysvars + stake config)
    let stake_account_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let _stake_history_info = next_account_info(account_info_iter)?;
    let _stake_config_info = next_account_info(account_info_iter)?;

    // other accounts
    // let _stake_authority_info = next_account_info(account_info_iter);

    let clock = &Clock::from_account_info(clock_info)?;
    // let stake_history = &StakeHistorySysvar(clock.epoch);

    // let vote_state = get_vote_state(vote_account_info)?;

    match get_stake_state(stake_account_info)? {
        StakeStateV2::Initialized(meta) => {
            let stake_amount = validate_delegated_amount(stake_account_info, &meta)?;

            let stake = new_stake(stake_amount, vote_account_info.key, clock.epoch);

            set_stake_state(
                stake_account_info,
                &StakeStateV2::Stake(meta, stake, StakeFlags::empty()),
            )
        }
        StakeStateV2::Stake(meta, mut stake, flags) => {
            let stake_amount = validate_delegated_amount(stake_account_info, &meta)?;

            redelegate_stake(&mut stake, stake_amount, vote_account_info.key, clock.epoch)?;

            set_stake_state(stake_account_info, &StakeStateV2::Stake(meta, stake, flags))
        }
        _ => panic!(), //Err(ProgramError::InvalidAccountData),
    }?;

    Ok(())
}

/// Model of `Split`: moves `split_lamports`, and the corresponding stake, to a
/// new stake account.
///
/// Accounts, in order: source stake account, destination stake account. The
/// model panics unless the destination's data is exactly
/// `StakeStateV2::size_of()` bytes and it is `Uninitialized`, and unless
/// `split_lamports` does not exceed the source's balance. Then, by the source's
/// state:
///
/// * `Stake`: either of the real program's two cases may happen, whatever the
///   balances:
///   * the source's delegated stake decreases by `split_lamports` minus its
///     rent-exempt reserve (saturating), and the destination's stake is that
///     same amount; or
///   * the source's delegated stake decreases by `split_lamports`, which
///     panics if less than the minimum delegation would remain, and the
///     destination's stake is `split_lamports` minus an arbitrary amount
///     (saturating).
///
///   The model panics if the destination's stake is below the minimum
///   delegation or the source's delegated stake is smaller than the decrease.
///   Both accounts are then `Stake` with the source's flags; the destination
///   has the source's delegation (voter and epochs) and meta, except for its
///   stake amount and an arbitrary rent-exempt reserve.
/// * `Initialized`: the destination becomes `Initialized` with the source's
///   meta and an arbitrary rent-exempt reserve.
/// * `Uninitialized`: the source must be a signer (otherwise the model
///   panics); the destination stays `Uninitialized`.
/// * Any other state panics.
///
/// If the whole balance is split, the source becomes `Uninitialized`. The
/// lamports are then moved with [`relocate_lamports`]. The stake authority and
/// the destination's rent exemption are not checked.
#[cvlr_early_panic::early_panic]
// The model keeps the stake program's former rent rule, the reserve recorded in
// `Meta`, rather than the current one, `Rent` read at each instruction.
#[allow(deprecated)]
pub fn process_split(accounts: &[AccountInfo], split_lamports: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 2 accounts
    let source_stake_account_info = next_account_info(account_info_iter)?;
    let destination_stake_account_info = next_account_info(account_info_iter)?;

    // other accounts
    // let _stake_authority_info = next_account_info(account_info_iter);

    // let clock = Clock::get()?;
    // let stake_history = &StakeHistorySysvar(clock.epoch);

    let destination_data_len = destination_stake_account_info.data_len();
    if destination_data_len != StakeStateV2::size_of() {
        panic!();
        // return Err(ProgramError::InvalidAccountData);
    }

    if let StakeStateV2::Uninitialized = get_stake_state(destination_stake_account_info)? {
        // we can split into this
    } else {
        panic!();
        // return Err(ProgramError::InvalidAccountData);
    }

    let source_lamport_balance = source_stake_account_info.lamports();
    let _destination_lamport_balance = destination_stake_account_info.lamports();

    if split_lamports > source_lamport_balance {
        panic!();
        // return Err(ProgramError::InsufficientFunds);
    }

    match get_stake_state(source_stake_account_info)? {
        StakeStateV2::Stake(source_meta, mut source_stake, stake_flags) => {
            let minimum_delegation = crate::get_minimum_delegation();

            // let is_active = get_effective_stake(&source_stake) > 0;

            // NOTE this function also internally summons Rent via syscall
            // let validated_split_info = validate_split_amount(
            //     source_lamport_balance,
            //     destination_lamport_balance,
            //     split_lamports,
            //     &source_meta,
            //     destination_data_len,
            //     minimum_delegation,
            //     is_active,
            // )?;

            // split the stake, subtract rent_exempt_balance unless
            // the destination account already has those lamports
            // in place.
            // this means that the new stake account will have a stake equivalent to
            // lamports minus rent_exempt_reserve if it starts out with a zero balance
            let (remaining_stake_delta, split_stake_amount) = if nondet::<u64>() == 0 {
                // If split amount equals the full source stake (as implied by 0
                // source_remaining_balance), the new split stake must equal the same
                // amount, regardless of any current lamport balance in the split account.
                // Since split accounts retain the state of their source account, this
                // prevents any magic activation of stake by pre-funding the split account.
                //
                // The new split stake also needs to ignore any positive delta between the
                // original rent_exempt_reserve and the split_rent_exempt_reserve, in order
                // to prevent magic activation of stake by splitting between accounts of
                // different sizes.
                let remaining_stake_delta =
                    split_lamports.saturating_sub(source_meta.rent_exempt_reserve);
                (remaining_stake_delta, remaining_stake_delta)
            } else {
                // Otherwise, the new split stake should reflect the entire split
                // requested, less any lamports needed to cover the
                // split_rent_exempt_reserve.
                if source_stake.delegation.stake.saturating_sub(split_lamports) < minimum_delegation
                {
                    panic!(); // return Err(StakeError::InsufficientDelegation.into());
                }

                (split_lamports, split_lamports.saturating_sub(nondet()))
            };

            if split_stake_amount < minimum_delegation {
                panic!(); // return Err(StakeError::InsufficientDelegation.into());
            }

            let destination_stake =
                source_stake.split(remaining_stake_delta, split_stake_amount)?;

            let mut destination_meta = source_meta;
            destination_meta.rent_exempt_reserve = nondet();

            set_stake_state(
                source_stake_account_info,
                &StakeStateV2::Stake(source_meta, source_stake, stake_flags),
            )?;

            set_stake_state(
                destination_stake_account_info,
                &StakeStateV2::Stake(destination_meta, destination_stake, stake_flags),
            )?;
        }
        StakeStateV2::Initialized(source_meta) => {
            // NOTE this function also internally summons Rent via syscall
            let mut destination_meta = source_meta;
            destination_meta.rent_exempt_reserve = nondet();

            set_stake_state(
                destination_stake_account_info,
                &StakeStateV2::Initialized(destination_meta),
            )?;
        }
        StakeStateV2::Uninitialized => {
            if !source_stake_account_info.is_signer {
                panic!();
                // return Err(ProgramError::MissingRequiredSignature);
            }
        }
        _ => panic!(), //return Err(ProgramError::InvalidAccountData),
    }

    // De-initialize state upon zero balance
    if split_lamports == source_lamport_balance {
        // set_stake_state(source_stake_account_info, &StakeStateV2::Uninitialized)?;
        write_uninitialized_stake(source_stake_account_info)?
    }

    relocate_lamports(
        source_stake_account_info,
        destination_stake_account_info,
        split_lamports,
    );

    Ok(())
}

/// Model of `Merge`: merges a source stake account into a destination.
///
/// Accounts, in order: destination stake account, source stake account,
/// `Clock` sysvar (unused), stake history (unused). The model panics if both
/// are the same account, or if either is not `Initialized` or `Stake`, and it
/// *assumes* both have the same authorities (executions where they differ are
/// dropped).
///
/// The destination becomes `Stake` with its own meta, an arbitrary `Stake`
/// (see [`nondet_stake`]) and no flags, whatever
/// the two accounts' delegations were: the merged delegation is not computed.
/// The source becomes `Uninitialized` and all its lamports move to the
/// destination. None of the real program's mergeability checks (activation
/// state, voter, credits) are made, and the stake authority is not checked.
#[cvlr_early_panic::early_panic]
pub fn process_merge(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // native asserts: 4 accounts (2 sysvars)
    let destination_stake_account_info = next_account_info(account_info_iter)?;
    let source_stake_account_info = next_account_info(account_info_iter)?;
    let _clock_info = next_account_info(account_info_iter)?;
    let _stake_history_info = next_account_info(account_info_iter)?;

    // other accounts
    // let _stake_authority_info = next_account_info(account_info_iter);

    // let clock = &Clock::from_account_info(clock_info)?;
    // let stake_history = &StakeHistorySysvar(clock.epoch);

    if source_stake_account_info.key == destination_stake_account_info.key {
        panic!();
        // return Err(ProgramError::InvalidArgument);
    }

    // msg!("Checking if destination stake is mergeable");
    // let destination_merge_kind = MergeKind::get_if_mergeable(
    //     &get_stake_state(destination_stake_account_info)?,
    //     destination_stake_account_info.lamports(),
    //     clock,
    //     stake_history,
    // )?;

    // Authorized staker is allowed to split/merge accounts
    // destination_merge_kind
    //     .meta()
    //     .authorized
    //     .check(&signers, StakeAuthorize::Staker)
    //     .map_err(|_| ProgramError::MissingRequiredSignature)?;

    // msg!("Checking if source stake is mergeable");
    // let source_merge_kind = MergeKind::get_if_mergeable(
    //     &get_stake_state(source_stake_account_info)?,
    //     source_stake_account_info.lamports(),
    //     clock,
    //     stake_history,
    // )?;

    // msg!("Merging stake accounts");
    // AG: this does not properly change the state of the destination account
    // if let Some(merged_state) = destination_merge_kind.merge(source_merge_kind, clock)? {
    // set_stake_state(destination_stake_account_info, &merged_state)?;
    // }

    // -- metas must match on authorized field for successful merge
    let source_meta: Meta = match get_stake_state(source_stake_account_info)? {
        StakeStateV2::Initialized(meta) | StakeStateV2::Stake(meta, _, _) => meta,
        _ =>
        // -- okay to panic because get_if_mergable returns an error in this case
        {
            panic!()
        }
    };
    let destination_meta: Meta = match get_stake_state(destination_stake_account_info)? {
        StakeStateV2::Initialized(meta) | StakeStateV2::Stake(meta, _, _) => meta,
        _ =>
        // -- okay to panic because get_if_mergable returns an error in this case
        {
            panic!()
        }
    };
    // -- below is checked by function metas_can_merge
    cvlr_assume!(destination_meta.authorized == source_meta.authorized);

    // -- reset destination stake except meta. This might be too abstract
    set_stake_state(
        destination_stake_account_info,
        &StakeStateV2::Stake(destination_meta, nondet_stake(), empty_stake_flags()),
    )?;

    // Source is about to be drained, de-initialize its state
    // set_stake_state(source_stake_account_info, &StakeStateV2::Uninitialized)?;
    write_uninitialized_stake(source_stake_account_info)?;

    // Drain the source stake account
    relocate_lamports(
        source_stake_account_info,
        destination_stake_account_info,
        source_stake_account_info.lamports(),
    );

    Ok(())
}
// The model keeps the stake program's former rent rule, the reserve recorded in
// `Meta`, rather than the current one, `Rent` read at each instruction.
#[allow(deprecated)]
pub(crate) fn validate_delegated_amount(
    account: &AccountInfo,
    meta: &Meta,
) -> Result<u64, ProgramError> {
    let stake_amount = account.lamports().saturating_sub(meta.rent_exempt_reserve); // can't stake the rent

    // Stake accounts may be initialized with a stake amount below the minimum
    // delegation so check that the minimum is met before delegation.
    if stake_amount < crate::get_minimum_delegation() {
        panic!();
    }
    Ok(stake_amount)
}

#[inline(always)]
pub(crate) fn new_stake(stake: u64, voter_pubkey: &Pubkey, activation_epoch: u64) -> Stake {
    Stake {
        delegation: Delegation::new(voter_pubkey, stake, activation_epoch),
        credits_observed: nondet(),
    }
}

pub(crate) fn redelegate_stake(
    stake: &mut Stake,
    stake_lamports: u64,
    voter_pubkey: &Pubkey,
    epoch: u64,
) -> Result<(), ProgramError> {
    // If stake is currently active:
    if get_effective_stake(stake) != 0 {
        // If pubkey of new voter is the same as current,
        // and we are scheduled to start deactivating this epoch,
        // we rescind deactivation
        if stake.delegation.voter_pubkey == *voter_pubkey
            && epoch == stake.delegation.deactivation_epoch
        {
            stake.delegation.deactivation_epoch = u64::MAX;
            return Ok(());
        } else {
            // can't redelegate to another pubkey if stake is active.
            panic!();
        }
    }
    // Either the stake is freshly activated, is active but has been
    // deactivated this epoch, or has fully de-activated.
    // Re-delegation implies either re-activation or un-deactivation

    stake.delegation.stake = stake_lamports;
    stake.delegation.activation_epoch = epoch;
    stake.delegation.deactivation_epoch = u64::MAX;
    stake.delegation.voter_pubkey = *voter_pubkey;
    stake.credits_observed = nondet(); // vote_state.credits();
    Ok(())
}

fn do_authorize(
    stake_account_info: &AccountInfo,
    new_authority: &Pubkey,
    authority_type: StakeAuthorize,
    _custodian: Option<&Pubkey>,
    _clock: &Clock,
) -> ProgramResult {
    match get_stake_state(stake_account_info)? {
        StakeStateV2::Initialized(mut meta) => {
            meta_authorized_authorize(&mut meta, new_authority, authority_type);
            set_stake_state(stake_account_info, &StakeStateV2::Initialized(meta))
        }
        StakeStateV2::Stake(mut meta, stake, stake_flags) => {
            meta_authorized_authorize(&mut meta, new_authority, authority_type);
            set_stake_state(
                stake_account_info,
                &StakeStateV2::Stake(meta, stake, stake_flags),
            )
        }
        _ => panic!(),
    }
}

#[inline(always)]
fn meta_authorized_authorize(
    meta: &mut Meta,
    new_authority: &Pubkey,
    authority_type: StakeAuthorize,
) {
    match authority_type {
        StakeAuthorize::Staker => meta.authorized.staker = *new_authority,
        StakeAuthorize::Withdrawer => meta.authorized.withdrawer = *new_authority,
    }
}

/// Moves `lamports` from `source_account_info` to `destination_account_info`.
///
/// Panics, ending the execution, if the source has fewer than `lamports`, if
/// the destination's balance would overflow, or if either account's lamports
/// are currently borrowed. Moving lamports from an account to itself leaves
/// its balance unchanged.
#[inline(always)]
pub fn relocate_lamports(
    source_account_info: &AccountInfo,
    destination_account_info: &AccountInfo,
    lamports: u64,
) {
    {
        let mut source_lamports = source_account_info.try_borrow_mut_lamports().unwrap();
        **source_lamports = source_lamports.checked_sub(lamports).unwrap();
    }

    {
        let mut destination_lamports = destination_account_info.try_borrow_mut_lamports().unwrap();
        **destination_lamports = destination_lamports.checked_add(lamports).unwrap();
    }
}

#[inline(always)]
fn get_stake_state(acc_info: &AccountInfo) -> Result<StakeStateV2, ProgramError> {
    Ok(stake_from_account_info_unchecked(acc_info))
}

#[inline(always)]
fn set_stake_state(acc: &AccountInfo, stake: &StakeStateV2) -> ProgramResult {
    stake_to_slice_unchecked(&mut acc.data.borrow_mut()[..], stake);
    Ok(())
}

/// Return non-deterministic amount in range
#[inline(always)]
fn get_effective_stake(stake: &Stake) -> u64 {
    let effective_stake: u64 = nondet();
    cvlr_assume!(effective_stake <= stake.delegation.stake);
    effective_stake
}

#[inline(always)]
fn write_uninitialized_stake(acc: &AccountInfo) -> ProgramResult {
    write_uninitialized_stake_to_slice(&mut acc.data.borrow_mut()[..]);
    Ok(())
}

#[inline(always)]
fn write_uninitialized_stake_to_slice(data: &mut [u8]) {
    cvlr_assume!(data.len() == StakeStateV2::size_of());
    let mut buf = data;
    buf.write_all(&0u32.to_le_bytes()).unwrap();
}
