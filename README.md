# Decentralized Voting - Anchor

This project is a decentralized voting system built on the Solana blockchain using the Anchor framework. Voters can cast votes for candidates, earn VOTE tokens as rewards, and deposit liquidity into a Raydium Automated Market Maker (AMM) pool. The program integrates with the SPL Token program for token management and Raydium for liquidity deposits.
---

## 📌 Features

1. **Voting** : 
    - Voters can vote for candidates, and their votes are recorded in a voting_state account.
2. **Token Rewards** : 
    - Voters receive 1 VOTE token as a reward for casting a vote.
3. **Liquidity Pool Integration** : 
    - Voters can deposit liquidity into a Raydium AMM pool during the voting process.
4. **End Voting** : 
    - The authority can end the voting session, preventing further votes.
5. **Results Retrieval** : 
    - The list of candidates and their respective vote counts can be retrieved after voting ends.

---



## 💡 Problems & Difficulties Faced

During the project development, the following challenges were encountered:

Challenges Faced
1. **Raydium CPI Testing** :
    - Faced errors when calling the raydium cpi call, so a mocking mechanism was implemented using the #[cfg(feature = "test")] flag. During testing, the Raydium CPI call is replaced with a mock log (msg!("Mocking Raydium CPI call");).
2. **Account Management** :
    - Managing multiple associated token accounts (e.g., voter_token_account, user_coin_token_account) and ensuring they are correctly initialized required careful planning.
3. **Testing Limitations** :
    - Testing the full functionality of the program required workarounds like mocking external dependencies (e.g., Raydium).  

---

## 📂 Project Structure

The program consists of the following key functions:

---

### 🔨 `initialize`
- Initializes the voting session by setting up candidates and storing the state in the `voting_state` account.  

---

### 🗳️ `cast_vote`
- Allows voters to cast votes for candidates.  
- Mints **1 VOTE token** as a reward to the voter's associated token account.  
- Deposits liquidity into the **Raydium AMM pool**.  

---

### 🛑 `end_voting`
- Ends the voting session, preventing further votes.  

---

### 📊 `get_results`
- Retrieves the list of candidates and their respective vote counts.  

---

## 🚀 How to Run the Program

### Prerequisites
- Install Rust: [Rust Installation Guide](https://www.rust-lang.org/tools/install)
- Install Solana CLI: [Solana CLI Installation](https://docs.solana.com/cli/install-solana-cli-tools)
- Install Anchor Framework: [Anchor Documentation](https://www.anchor-lang.com/)

### Steps to Run
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/voting-system.git
   cd voting-system

2. Install dependencies:
   ```bash
   cargo build
   yarn install

3. Configure Solana CLI for Devnet:
    ```bash
    solana config set --url https://api.devnet.solana.com
    solana airdrop 2

4. Deploy the program and run test:
    ```bash
    anchor deploy
    anchor test -- --features "test"


