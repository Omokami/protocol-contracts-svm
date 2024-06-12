/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  Account,
  Address,
  Codec,
  Decoder,
  EncodedAccount,
  Encoder,
  FetchAccountConfig,
  FetchAccountsConfig,
  MaybeAccount,
  MaybeEncodedAccount,
  ReadonlyUint8Array,
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  fixDecoderSize,
  fixEncoderSize,
  getAddressDecoder,
  getAddressEncoder,
  getBytesDecoder,
  getBytesEncoder,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU32Encoder,
  transformEncoder,
} from '@solana/web3.js';

export type TokenGroupMember = {
  discriminator: ReadonlyUint8Array;
  /**
   * The associated mint, used to counter spoofing to be sure that member
   * belongs to a particular mint
   */
  mint: Address;
  /** The pubkey of the `TokenGroup` */
  group: Address;
  /** The member number */
  memberNumber: number;
};

export type TokenGroupMemberArgs = {
  /**
   * The associated mint, used to counter spoofing to be sure that member
   * belongs to a particular mint
   */
  mint: Address;
  /** The pubkey of the `TokenGroup` */
  group: Address;
  /** The member number */
  memberNumber: number;
};

export function getTokenGroupMemberEncoder(): Encoder<TokenGroupMemberArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', fixEncoderSize(getBytesEncoder(), 8)],
      ['mint', getAddressEncoder()],
      ['group', getAddressEncoder()],
      ['memberNumber', getU32Encoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: new Uint8Array([17, 208, 50, 173, 30, 127, 245, 94]),
    })
  );
}

export function getTokenGroupMemberDecoder(): Decoder<TokenGroupMember> {
  return getStructDecoder([
    ['discriminator', fixDecoderSize(getBytesDecoder(), 8)],
    ['mint', getAddressDecoder()],
    ['group', getAddressDecoder()],
    ['memberNumber', getU32Decoder()],
  ]);
}

export function getTokenGroupMemberCodec(): Codec<
  TokenGroupMemberArgs,
  TokenGroupMember
> {
  return combineCodec(
    getTokenGroupMemberEncoder(),
    getTokenGroupMemberDecoder()
  );
}

export function decodeTokenGroupMember<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<TokenGroupMember, TAddress>;
export function decodeTokenGroupMember<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<TokenGroupMember, TAddress>;
export function decodeTokenGroupMember<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<TokenGroupMember, TAddress>
  | MaybeAccount<TokenGroupMember, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getTokenGroupMemberDecoder()
  );
}

export async function fetchTokenGroupMember<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<TokenGroupMember, TAddress>> {
  const maybeAccount = await fetchMaybeTokenGroupMember(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeTokenGroupMember<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<TokenGroupMember, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeTokenGroupMember(maybeAccount);
}

export async function fetchAllTokenGroupMember(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<TokenGroupMember>[]> {
  const maybeAccounts = await fetchAllMaybeTokenGroupMember(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeTokenGroupMember(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<TokenGroupMember>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeTokenGroupMember(maybeAccount)
  );
}
