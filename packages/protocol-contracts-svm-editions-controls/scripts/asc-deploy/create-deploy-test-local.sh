npx ts-node ./src/cli/createDeployment.ts \
  -t 7jB2kzg5FbuNjETEgjnERfznGMFs7sQ7nhoXcJzwJpxj \
  -k ~/.config/solana/id.json \
  -r https://testnet.dev2.eclipsenetwork.xyz \
  -s ASC2 \
  --maxNumberOfTokens 10000 \
  --maxMintsPerWallet 300 \
  -u "https://ipfs.raribleuserdata.com/ipfs/QmYGkcfE9Zzm71wiKks8tWC3hkgc45vKWaod6LvEYdCHdG" \
  -n "AFTER SCHOOL CLUB" \
  --creators 7jB2kzg5FbuNjETEgjnERfznGMFs7sQ7nhoXcJzwJpxj:100 \
  --royaltyBasisPoints 500 \
  --platformFeeValue 650000 \
  --platformFeeRecipients AsSKqK7CkxFUf3KaoQzzr8ZLPm5fFguUtVE5QwGALQQn:100 \
  --isFeeFlat \
  --itemBaseUri "https://rarible-drops.s3.filebase.com/Eclipse/asc/metadata/metadata_src/{}.json" \
  --itemBaseName "ASC #{}"

npx ts-node ./src/cli/addPhase.ts \
  -d F5PSjXHyAsJWgL8MakvK1WnoMbzRSdNQqrAbd2MQduFw \
  -k ~/.config/solana/id.json \
  -r https://testnet.dev2.eclipsenetwork.xyz \
  --maxMintsPerWallet 300 \
  --maxMintsTotal 300 \
  --priceAmount 0 \
  -s 1732105800 \
  -e 1732111200

npx ts-node ./src/cli/addPhase.ts \
  -d F5PSjXHyAsJWgL8MakvK1WnoMbzRSdNQqrAbd2MQduFw \
  -k ~/.config/solana/id.json \
  -r https://testnet.dev2.eclipsenetwork.xyz \
  --maxMintsPerWallet 300 \
  --maxMintsTotal 10000 \
  --priceAmount 3200000 \
  -s 1732111200 \
  -e 1732197600

npx ts-node ./src/cli/addPhase.ts \
  -d F5PSjXHyAsJWgL8MakvK1WnoMbzRSdNQqrAbd2MQduFw \
  -k ~/.config/solana/id.json \
  -r https://testnet.dev2.eclipsenetwork.xyz \
  --maxMintsPerWallet 300 \
  --maxMintsTotal 3200000 \
  --priceAmount 3200000 \
  -s 1732197600 \
  -e 1732370400