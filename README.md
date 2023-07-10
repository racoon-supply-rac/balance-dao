# Balance Swap

Contract to swap a native IBC asset for a newly minted TokenFactory asset.

## Details
This contract was made for BalanceDAO to swap $JUNO for a newly minted $BALANCE token on Juno Network.

The contract does one thing:
- Swaps $JUNO for $BALANCE at a fixed rate of `21000000 / 185562268 = 0.113169558802763`

If you send 1 $JUNO along with a `swap` message, it will:
- Burn X% of the $JUNO received
- Send Y% of the $JUNO received to the development fund
- Send Z% of the $JUNO received to the vesting fund
- Send U% of the $JUNO to the developer who made the current contract
- Send 0.113169 $BALANCE to the sender

The X, Y, Z, U variables are defined when the contract is instantiated.

Additionally, the contract takes care of creating the TokenFactory denom and mints them 
when a swap happens.
