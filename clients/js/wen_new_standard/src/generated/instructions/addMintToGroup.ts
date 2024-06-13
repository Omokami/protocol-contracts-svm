/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  fixDecoderSize,
  fixEncoderSize,
  getBytesDecoder,
  getBytesEncoder,
  getStructDecoder,
  getStructEncoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type ReadonlyUint8Array,
  type TransactionSigner,
  type WritableAccount,
  type WritableSignerAccount,
} from '@solana/web3.js';
import { WEN_NEW_STANDARD_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type AddMintToGroupInstruction<
  TProgram extends string = typeof WEN_NEW_STANDARD_PROGRAM_ADDRESS,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountGroup extends string | IAccountMeta<string> = string,
  TAccountMember extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountManager extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountPayer extends string
        ? WritableSignerAccount<TAccountPayer> &
            IAccountSignerMeta<TAccountPayer>
        : TAccountPayer,
      TAccountAuthority extends string
        ? ReadonlySignerAccount<TAccountAuthority> &
            IAccountSignerMeta<TAccountAuthority>
        : TAccountAuthority,
      TAccountGroup extends string
        ? WritableAccount<TAccountGroup>
        : TAccountGroup,
      TAccountMember extends string
        ? WritableAccount<TAccountMember>
        : TAccountMember,
      TAccountMint extends string
        ? WritableAccount<TAccountMint>
        : TAccountMint,
      TAccountManager extends string
        ? ReadonlyAccount<TAccountManager>
        : TAccountManager,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type AddMintToGroupInstructionData = {
  discriminator: ReadonlyUint8Array;
};

export type AddMintToGroupInstructionDataArgs = {};

export function getAddMintToGroupInstructionDataEncoder(): Encoder<AddMintToGroupInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', fixEncoderSize(getBytesEncoder(), 8)]]),
    (value) => ({
      ...value,
      discriminator: new Uint8Array([236, 25, 99, 48, 185, 60, 235, 112]),
    })
  );
}

export function getAddMintToGroupInstructionDataDecoder(): Decoder<AddMintToGroupInstructionData> {
  return getStructDecoder([
    ['discriminator', fixDecoderSize(getBytesDecoder(), 8)],
  ]);
}

export function getAddMintToGroupInstructionDataCodec(): Codec<
  AddMintToGroupInstructionDataArgs,
  AddMintToGroupInstructionData
> {
  return combineCodec(
    getAddMintToGroupInstructionDataEncoder(),
    getAddMintToGroupInstructionDataDecoder()
  );
}

export type AddMintToGroupInput<
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountGroup extends string = string,
  TAccountMember extends string = string,
  TAccountMint extends string = string,
  TAccountManager extends string = string,
  TAccountSystemProgram extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  payer: TransactionSigner<TAccountPayer>;
  authority: TransactionSigner<TAccountAuthority>;
  group: Address<TAccountGroup>;
  member: Address<TAccountMember>;
  mint: Address<TAccountMint>;
  manager: Address<TAccountManager>;
  systemProgram?: Address<TAccountSystemProgram>;
  tokenProgram?: Address<TAccountTokenProgram>;
};

export function getAddMintToGroupInstruction<
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountGroup extends string,
  TAccountMember extends string,
  TAccountMint extends string,
  TAccountManager extends string,
  TAccountSystemProgram extends string,
  TAccountTokenProgram extends string,
>(
  input: AddMintToGroupInput<
    TAccountPayer,
    TAccountAuthority,
    TAccountGroup,
    TAccountMember,
    TAccountMint,
    TAccountManager,
    TAccountSystemProgram,
    TAccountTokenProgram
  >
): AddMintToGroupInstruction<
  typeof WEN_NEW_STANDARD_PROGRAM_ADDRESS,
  TAccountPayer,
  TAccountAuthority,
  TAccountGroup,
  TAccountMember,
  TAccountMint,
  TAccountManager,
  TAccountSystemProgram,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = WEN_NEW_STANDARD_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    payer: { value: input.payer ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    group: { value: input.group ?? null, isWritable: true },
    member: { value: input.member ?? null, isWritable: true },
    mint: { value: input.mint ?? null, isWritable: true },
    manager: { value: input.manager ?? null, isWritable: false },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb' as Address<'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.group),
      getAccountMeta(accounts.member),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.manager),
      getAccountMeta(accounts.systemProgram),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getAddMintToGroupInstructionDataEncoder().encode({}),
  } as AddMintToGroupInstruction<
    typeof WEN_NEW_STANDARD_PROGRAM_ADDRESS,
    TAccountPayer,
    TAccountAuthority,
    TAccountGroup,
    TAccountMember,
    TAccountMint,
    TAccountManager,
    TAccountSystemProgram,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedAddMintToGroupInstruction<
  TProgram extends string = typeof WEN_NEW_STANDARD_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    payer: TAccountMetas[0];
    authority: TAccountMetas[1];
    group: TAccountMetas[2];
    member: TAccountMetas[3];
    mint: TAccountMetas[4];
    manager: TAccountMetas[5];
    systemProgram: TAccountMetas[6];
    tokenProgram: TAccountMetas[7];
  };
  data: AddMintToGroupInstructionData;
};

export function parseAddMintToGroupInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedAddMintToGroupInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 8) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      payer: getNextAccount(),
      authority: getNextAccount(),
      group: getNextAccount(),
      member: getNextAccount(),
      mint: getNextAccount(),
      manager: getNextAccount(),
      systemProgram: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getAddMintToGroupInstructionDataDecoder().decode(instruction.data),
  };
}
