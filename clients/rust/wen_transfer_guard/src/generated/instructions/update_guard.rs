//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use crate::generated::types::CpiRule;
use crate::generated::types::MetadataAdditionalFieldRule;
use crate::generated::types::TransferAmountRule;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct UpdateGuard {
    pub guard: solana_program::pubkey::Pubkey,

    pub mint: solana_program::pubkey::Pubkey,

    pub token_account: solana_program::pubkey::Pubkey,

    pub guard_authority: solana_program::pubkey::Pubkey,

    pub token_program: solana_program::pubkey::Pubkey,

    pub system_program: solana_program::pubkey::Pubkey,
}

impl UpdateGuard {
    pub fn instruction(
        &self,
        args: UpdateGuardInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UpdateGuardInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.guard, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.guard_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UpdateGuardInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::WEN_TRANSFER_GUARD_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UpdateGuardInstructionData {
    discriminator: [u8; 8],
}

impl UpdateGuardInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: [51, 38, 175, 180, 25, 249, 39, 24],
        }
    }
}

impl Default for UpdateGuardInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateGuardInstructionArgs {
    pub cpi_rule: Option<CpiRule>,
    pub transfer_amount_rule: Option<TransferAmountRule>,
    pub additional_fields_rule: Vec<MetadataAdditionalFieldRule>,
}

/// Instruction builder for `UpdateGuard`.
///
/// ### Accounts:
///
///   0. `[writable]` guard
///   1. `[]` mint
///   2. `[]` token_account
///   3. `[signer]` guard_authority
///   4. `[optional]` token_program (default to `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`)
///   5. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct UpdateGuardBuilder {
    guard: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    token_account: Option<solana_program::pubkey::Pubkey>,
    guard_authority: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    cpi_rule: Option<CpiRule>,
    transfer_amount_rule: Option<TransferAmountRule>,
    additional_fields_rule: Option<Vec<MetadataAdditionalFieldRule>>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdateGuardBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn guard(&mut self, guard: solana_program::pubkey::Pubkey) -> &mut Self {
        self.guard = Some(guard);
        self
    }
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    #[inline(always)]
    pub fn token_account(&mut self, token_account: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_account = Some(token_account);
        self
    }
    #[inline(always)]
    pub fn guard_authority(
        &mut self,
        guard_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.guard_authority = Some(guard_authority);
        self
    }
    /// `[optional account, default to 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb']`
    #[inline(always)]
    pub fn token_program(&mut self, token_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    /// `[optional argument]`
    #[inline(always)]
    pub fn cpi_rule(&mut self, cpi_rule: CpiRule) -> &mut Self {
        self.cpi_rule = Some(cpi_rule);
        self
    }
    /// `[optional argument]`
    #[inline(always)]
    pub fn transfer_amount_rule(&mut self, transfer_amount_rule: TransferAmountRule) -> &mut Self {
        self.transfer_amount_rule = Some(transfer_amount_rule);
        self
    }
    #[inline(always)]
    pub fn additional_fields_rule(
        &mut self,
        additional_fields_rule: Vec<MetadataAdditionalFieldRule>,
    ) -> &mut Self {
        self.additional_fields_rule = Some(additional_fields_rule);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = UpdateGuard {
            guard: self.guard.expect("guard is not set"),
            mint: self.mint.expect("mint is not set"),
            token_account: self.token_account.expect("token_account is not set"),
            guard_authority: self.guard_authority.expect("guard_authority is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
            )),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };
        let args = UpdateGuardInstructionArgs {
            cpi_rule: self.cpi_rule.clone(),
            transfer_amount_rule: self.transfer_amount_rule.clone(),
            additional_fields_rule: self
                .additional_fields_rule
                .clone()
                .expect("additional_fields_rule is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `update_guard` CPI accounts.
pub struct UpdateGuardCpiAccounts<'a, 'b> {
    pub guard: &'b solana_program::account_info::AccountInfo<'a>,

    pub mint: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub guard_authority: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `update_guard` CPI instruction.
pub struct UpdateGuardCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub guard: &'b solana_program::account_info::AccountInfo<'a>,

    pub mint: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub guard_authority: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UpdateGuardInstructionArgs,
}

impl<'a, 'b> UpdateGuardCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: UpdateGuardCpiAccounts<'a, 'b>,
        args: UpdateGuardInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            guard: accounts.guard,
            mint: accounts.mint,
            token_account: accounts.token_account,
            guard_authority: accounts.guard_authority,
            token_program: accounts.token_program,
            system_program: accounts.system_program,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.guard.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.guard_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_program.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = UpdateGuardInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::WEN_TRANSFER_GUARD_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.guard.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.token_account.clone());
        account_infos.push(self.guard_authority.clone());
        account_infos.push(self.token_program.clone());
        account_infos.push(self.system_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `UpdateGuard` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` guard
///   1. `[]` mint
///   2. `[]` token_account
///   3. `[signer]` guard_authority
///   4. `[]` token_program
///   5. `[]` system_program
#[derive(Clone, Debug)]
pub struct UpdateGuardCpiBuilder<'a, 'b> {
    instruction: Box<UpdateGuardCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdateGuardCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UpdateGuardCpiBuilderInstruction {
            __program: program,
            guard: None,
            mint: None,
            token_account: None,
            guard_authority: None,
            token_program: None,
            system_program: None,
            cpi_rule: None,
            transfer_amount_rule: None,
            additional_fields_rule: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn guard(&mut self, guard: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.guard = Some(guard);
        self
    }
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    #[inline(always)]
    pub fn token_account(
        &mut self,
        token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_account = Some(token_account);
        self
    }
    #[inline(always)]
    pub fn guard_authority(
        &mut self,
        guard_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.guard_authority = Some(guard_authority);
        self
    }
    #[inline(always)]
    pub fn token_program(
        &mut self,
        token_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    /// `[optional argument]`
    #[inline(always)]
    pub fn cpi_rule(&mut self, cpi_rule: CpiRule) -> &mut Self {
        self.instruction.cpi_rule = Some(cpi_rule);
        self
    }
    /// `[optional argument]`
    #[inline(always)]
    pub fn transfer_amount_rule(&mut self, transfer_amount_rule: TransferAmountRule) -> &mut Self {
        self.instruction.transfer_amount_rule = Some(transfer_amount_rule);
        self
    }
    #[inline(always)]
    pub fn additional_fields_rule(
        &mut self,
        additional_fields_rule: Vec<MetadataAdditionalFieldRule>,
    ) -> &mut Self {
        self.instruction.additional_fields_rule = Some(additional_fields_rule);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = UpdateGuardInstructionArgs {
            cpi_rule: self.instruction.cpi_rule.clone(),
            transfer_amount_rule: self.instruction.transfer_amount_rule.clone(),
            additional_fields_rule: self
                .instruction
                .additional_fields_rule
                .clone()
                .expect("additional_fields_rule is not set"),
        };
        let instruction = UpdateGuardCpi {
            __program: self.instruction.__program,

            guard: self.instruction.guard.expect("guard is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            token_account: self
                .instruction
                .token_account
                .expect("token_account is not set"),

            guard_authority: self
                .instruction
                .guard_authority
                .expect("guard_authority is not set"),

            token_program: self
                .instruction
                .token_program
                .expect("token_program is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UpdateGuardCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    guard: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    guard_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    cpi_rule: Option<CpiRule>,
    transfer_amount_rule: Option<TransferAmountRule>,
    additional_fields_rule: Option<Vec<MetadataAdditionalFieldRule>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}