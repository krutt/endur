# Endur

[![Bitcoin-only](https://img.shields.io/badge/bitcoin-only-FF9900?logo=bitcoin)](https://twentyone.world)
[![License: GNU GPLv3](https://img.shields.io/badge/License-GNU%20GPLv3-yellow.svg)](https://github.com/krutt/endur/blob/master/LICENSE)
[![Top](https://img.shields.io/github/languages/top/krutt/endur)](https://github.com/krutt/endur)
[![Languages](https://img.shields.io/github/languages/count/krutt/smatt)](https://github.com/krutt/endur)
[![Size](https://img.shields.io/github/repo-size/krutt/endur)](https://github.com/krutt/smatt)
[![Last commit](https://img.shields.io/github/last-commit/krutt/endur/master)](https://github.com/krutt/endur)

[![Endur banner](https://github.com/krutt/endur/blob/master/static/endur-banner.svg)](static/endur-banner.svg)

### What are Stable Channels?

Stable Channels is a library and technology that lets users peg a portion of their bitcoin to a dollar balance. 

The Stable Channels implementation runs a full Lightning node that lets users stabilize their bitcoin holdings. 

The LSP/Server keeps the user stable by overcollateralizing the Stable Channel at the time of channel creation. Each user (the stability receiver and the the LSP/stability provider) puts in the same amount of bitcoin, the stability mechanism is activated. 

The stability mechanism works like this: each node queries four exchange price feeds every minute. Based on the updated price, they adjust the channel balance with their counterparty to keep the stability receiver's balance at a fixed dollar value (e.g., $100,000 of bitcoin).

<p align="center">
  <img src="./sc.gif" alt="Stable Channels Architecture" width="700"/>
</p>

Both parties remain self-custodial and can opt out anytime via cooperative or forced on-chain channel closure. 

The project is in-progress and is based on LDK Node. Prior versions were compatible with LND and CLN. These legacy implementation can be found in /legacy. 

Links with examples:
- **Basic example:** [Twitter thread](https://x.com/tonklaus/status/1729567459579945017)
- **In-depth discussion:** [Delving Bitcoin](https://delvingbitcoin.org/t/stable-channels-peer-to-peer-dollar-balances-on-lightning)
- **Project website:** [StableChannels.com](https://www.stablechannels.com)


### Stable Channels Process

Every 30 seconds, the price of bitcoin:

- **(a) Goes up:**
  - **Stable Receiver loses bitcoin.**
    - Less bitcoin is needed to maintain the dollar value.
    - The Stable Receiver pays the Stable Provider.
  
- **(b) Goes down:**
  - **Stable Receiver gains bitcoin.**
    - More bitcoin is needed to maintain the dollar value.
    - The Stable Provider pays the Stable Receiver.
  
- **(c) Stays the same:**
  - **No action required.**

## Contributions

### Prerequisites

* [git](https://git-scm.com/) - --fast-version-control
* [python](https://www.python.org) 3.9 and above - High-level general-purpose programming language
* [uv](https://docs.astral.sh/uv) - Extremely fast Python package & project manager, written in Rust

The following guide walks through setting up your local working environment using `git`
as distributed version control system and `uv` as Python package and version manager.
If you do not have `git` installed, run the following command.

<details>
  <summary> Install using Homebrew (Darwin) </summary>
  
  ```bash
  brew install git
  ```
</details>

<details>
  <summary> Install via binary installer (Linux or Windows Subsystem for Linux [WSL]) </summary>
  
  * Debian-based package management
  ```bash
  sudo apt install git-all
  ```

  * Fedora-based package management
  ```bash
  sudo dnf install git-all
  ```
</details>

<details>
  <summary> Install using Winget (Windows Powershell) </summary>
  
  ```bash
  winget install --id Git.Git -e --source winget
  ```
</details>

If you do not have `uv` installed, run the following command.

<details>
  <summary> Install using Homebrew (Darwin) </summary>

  ```bash
  brew install uv
  ```
</details>

<details>
  <summary>
    Install using standalone installer (Darwin, Linux, or Windows Subsystem for Linux [WSL])
  </summary>

  ```bash
  curl -LsSf https://astral.sh/uv/install.sh | sh
  ```
</details>

<details>
  <summary> Install using Winget (Windows Powershell) </summary>

  ```bash
  winget install --id=astral-sh.uv -e
  ```
</details>

Once you have `git` distributed version control system installed, you can
clone the current repository and  install any version of Python above version
3.9 for this project. The following commands help you set up and activate a
Python virtual environment where `uv` can download project dependencies from the `PyPI`
open-sourced registry defined under `pyproject.toml` file.

<details>
  <summary> Set up environment and synchronize project dependencies </summary>

  ```bash
  git clone git@github.com:krutt/endur.git
  cd endur
  uv venv --python 3.9.6
  source .venv/bin/activate
  uv sync --dev
  ```
</details>

## Payout Examples (entry = $100,000/BTC)

Each side puts in 1 BTC at $100,0000.

Abbreviations:
- SR = Stable Receiver (targeting $100,000)
- SP = Stable Provider
- Δ = Delta / Change

| Price Change (%) | New BTC Price | SR (BTC) | SR (USD) | SP (BTC) | SP (USD) | SR Fiat Δ$ | SR BTC Δ | SR Fiat Δ% | SR BTC Δ% | SP Fiat Δ$ | SP BTC Δ | SP Fiat Δ% | SP BTC Δ% |
|------------------|---------------|----------|----------|----------|----------|------------|----------|------------|-----------|------------|----------|------------|-----------|
| -30              | 70,000.00     | 1.43     | 100,000.00| 0.57    | 40,000.00| 0.00       | +0.43    | 0%         | +42.86%   | -60,000.00 | -0.43    | -60.00%    | -42.86%   |
| -20              | 80,000.00     | 1.25     | 100,000.00| 0.75    | 60,000.00| 0.00       | +0.25    | 0%         | +25.00%   | -40,000.00 | -0.25    | -40.00%    | -25.00%   |
| -10              | 90,000.00     | 1.11     | 100,000.00| 0.89    | 80,000.00| 0.00       | +0.11    | 0%         | +11.11%   | -20,000.00 | -0.11    | -20.00%    | -11.11%   |
| 0                | 100,000.00    | 1.00     | 100,000.00| 1.00    | 100,000.00| 0.00      | 0.00     | 0%         | 0%        | 0.00       | 0.00     | 0%         | 0%        |
| 10               | 110,000.00    | 0.91     | 100,000.00| 1.09    | 120,000.00| 0.00      | -0.09    | 0%         | -9.09%    | +20,000.00 | +0.09    | +20.00%    | +9.09%    |
| 20               | 120,000.00    | 0.83     | 100,000.00| 1.17    | 140,000.00| 0.00      | -0.17    | 0%         | -16.67%   | +40,000.00 | +0.17    | +40.00%    | +16.67%   |
| 30               | 130,000.00    | 0.77     | 100,000.00| 1.23    | 160,000.00| 0.00      | -0.23    | 0%         | -23.08%   | +60,000.00 | +0.23    | +60.00%    | +23.08%   |

### Acknowledgements

1. [Satoshi Lives!](https://github.com/bitcoin/bitcoin)
2. [The Bitcoin Lightning Network: Scalable Off-Chain Instant Payments](https://lightning.network/lightning-network-paper.pdf)
3. [Delving Bitcoin: Stable Channels - peer-to-peer dollar balances on Lightning](https://delvingbitcoin.org/t/stable-channels-peer-to-peer-dollar-balances-on-lightning/875)
4. [stable-channels](https://github.com/toneloc/stable-channels) 
5. [หลายขนาด - Lai Khnad](https://www.f0nt.com/release/sov-laikhnad) typeface
  by [uvSOV - Worawut Thanawatanawanich](https://fb.com/worawut.thanawatanawanich)
6. [Chaumian E-cash](...)
7. [Cashu](...)

## License

This project is licensed under the terms of the GNU General Public License 3.0 license.
