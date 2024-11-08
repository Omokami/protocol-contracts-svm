export ANCHOR_WALLET=~/.config/solana/id.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

npx ts-node ./src/initMarket.ts \
  -k ~/.config/solana/id.json \
  -r https://api.devnet.solana.com \
  -m "674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw" \
  --feeBps 250 \
  --feeRecipient 674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw
  # --ledger
