/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  fixEncoderSize,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
  type Address,
  type ProgramDerivedAddress,
} from '@solana/web3.js';

export type ExtraMetasAccountSeeds = {
  mint: Address;
};

export async function findExtraMetasAccountPda(
  seeds: ExtraMetasAccountSeeds,
  config: { programAddress?: Address | undefined } = {}
): Promise<ProgramDerivedAddress> {
  const {
    programAddress = 'wns1gDLt8fgLcGhWi5MqAqgXpwEP1JftKE9eZnXS1HM' as Address<'wns1gDLt8fgLcGhWi5MqAqgXpwEP1JftKE9eZnXS1HM'>,
  } = config;
  return await getProgramDerivedAddress({
    programAddress,
    seeds: [
      fixEncoderSize(getBytesEncoder(), 19).encode(
        new Uint8Array([
          101, 120, 116, 114, 97, 45, 97, 99, 99, 111, 117, 110, 116, 45, 109,
          101, 116, 97, 115,
        ])
      ),
      getAddressEncoder().encode(seeds.mint),
    ],
  });
}
