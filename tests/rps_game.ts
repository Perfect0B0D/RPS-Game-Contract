import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RpsGame } from "../target/types/rps_game";

import {
  SystemProgram,
  Keypair,
  PublicKey,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";


describe("rps_game", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  const program = anchor.workspace.RpsGame as Program<RpsGame>;

  let globalState, vault, round: PublicKey;
  let globalStateBump, vaultBump, roundBump: number;

  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const ROUND_STATE_SEED = "ROUND-STATE-SEED";
  const VAULT_SEED = "VAULT-SEED";

  let owner = Keypair.fromSecretKey(
    Uint8Array.from(/* owner address */)
  );

  let user = Keypair.fromSecretKey(
    Uint8Array.from(/* user address */)
  );

  it("is initialized accounts", async () => {
    [globalState, globalStateBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_STATE_SEED)],
        program.programId
      );

    [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(VAULT_SEED)],
      program.programId
    );
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const fee = 25; // (2.5%)

    const tx = await program.rpc.initialize(new anchor.BN(fee), {
      accounts: {
        owner: owner.publicKey,
        globalState,
        vault,
        systemProgram: SystemProgram.programId,
      },
      signers: [owner],
    });
    console.log("Your transaction signature", tx);
    const globalStateData = await program.account.globalState.fetch(
      globalState
    );
    console.log("globalStateData->", globalStateData);
  });

  it("update owner", async() => {
    const new_owner = new PublicKey("JE3n8Ch8s9QGd25yUKRkhCcdRNUEdhG9BFWSSt5h9wF4");

    const tx = await program.rpc.updateOwner(
      new_owner,
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      }
    );
    const globalStateData = await program.account.globalState.fetch(globalState);
    console.log("updated owner",globalStateData.owner.toString());
  });

  it("update fee", async () => {
    const fee = 26; // (2.6%)
    const tx = await program.rpc.updateFee(new anchor.BN(fee), {
      accounts: {
        owner: owner.publicKey,
        globalState,
        systemProgram: SystemProgram.programId,
      },
      signers: [owner],
    });
  });

  it("create round", async () => {
    const depositAmount = 5000000;
    const roundIndex = 1;
    try {
      // Round 1
      [round, roundBump] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(ROUND_STATE_SEED),
          new anchor.BN(roundIndex).toBuffer("le", 4),
          owner.publicKey.toBuffer(),
        ],
        program.programId
      );
      const tx = await program.rpc.createRound(
        roundIndex,
        new anchor.BN(depositAmount),
        {
          accounts: {
            user: owner.publicKey,
            globalState,
            vault,
            round,
            systemProgram: SystemProgram.programId,
          },
          signers: [owner],
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
      const globalStateData = await program.account.globalState.fetch(
        globalState
      );
      console.log("globalStateData->", globalStateData);
    } catch (error) {
      console.log(error);
    }
  });

  it("join round", async () => {
    const roundIndex = 1;
    try {
      // Round 1
      [round, roundBump] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(ROUND_STATE_SEED),
          new anchor.BN(roundIndex).toBuffer("le", 4),
          owner.publicKey.toBuffer(),
        ],
        program.programId
      );
      const tx = await program.rpc.joinRound(
        roundIndex,
        {
          accounts: {
            user: user.publicKey,
            globalState,
            vault,
            round,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [user],
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
    } catch (error) {
      console.log(error);
    }
  });

  it("play test", async () => {
    const roundIndex = 1;
    const isCreator = true;
    const rock = 1;
    const paper = 2;
    const scissors  = 3;
    try {
      [round, roundBump] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(ROUND_STATE_SEED),
          new anchor.BN(roundIndex).toBuffer("le", 4),
          owner.publicKey.toBuffer(),
        ],
        program.programId
      );
      const tx1 = await program.rpc.play(
        roundIndex,
        true,
        rock,
        {
          accounts: {
            user: owner.publicKey,
            globalState,
            round,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [owner],
        }
      );
      const tx2 = await program.rpc.play(
        roundIndex,
        false,
        paper,
        {
          accounts: {
            user: user.publicKey,
            globalState,
            round,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [user],
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
    } catch (error) {
      console.log(error);
    }
  });

  it("claim", async () => {
    try {
      const roundIndex = 1;
      const isCreator = false;
  
      [round, roundBump] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(ROUND_STATE_SEED),
          new anchor.BN(roundIndex).toBuffer("le", 4),
          owner.publicKey.toBuffer(),
        ],
        program.programId
      );
  
      let balance = await program.provider.connection.getBalance(vault);
      let lamportBalance = balance / 1000000000;
      console.log("lamportBalance before withdraw->", lamportBalance);
  
      const tx = await program.rpc.claim(roundIndex, isCreator, {
        accounts: {
          user: user.publicKey,
          owner: owner.publicKey,
          globalState,
          vault,
          round,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        },
        signers: [user],
      });
  
      balance = await program.provider.connection.getBalance(vault);
      lamportBalance = balance / 1000000000;
      console.log("lamportBalance after withdraw->", lamportBalance);
    } catch (error) {
      console.log(error)
    }

  });
});
