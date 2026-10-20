//! Which Zcash network FERRY talks to.

use zcash_protocol::consensus::{self, BlockHeight, NetworkType, NetworkUpgrade, Parameters};
use zcash_protocol::local_consensus::LocalNetwork;

/// The network a FERRY wallet belongs to.
///
/// Regtest carries its own activation heights, because a local chain chooses them. They must
/// match the node's configuration exactly (`regtest/heights.env` renders both from one source).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FerryNetwork {
    Main,
    Test,
    Regtest(LocalNetwork),
}

impl FerryNetwork {
    /// A regtest network where every upgrade up to NU6.2 activates at `early_upgrades` and NU6.3
    /// (the Ironwood pool) at `nu6_3`, matching `regtest/heights.env`.
    pub fn regtest(early_upgrades: u32, nu6_3: u32) -> Self {
        let early = Some(BlockHeight::from_u32(early_upgrades));
        FerryNetwork::Regtest(LocalNetwork {
            overwinter: early,
            sapling: early,
            blossom: early,
            heartwood: early,
            canopy: early,
            nu5: early,
            nu6: early,
            nu6_1: early,
            nu6_2: early,
            nu6_3: Some(BlockHeight::from_u32(nu6_3)),
            nu7: None,
            #[cfg(zcash_unstable = "nutachyon")]
            nu_tachyon: None,
        })
    }

    /// The height at which the wallet's chain gained the Ironwood pool, if it has.
    pub fn ironwood_activation(&self) -> Option<BlockHeight> {
        self.activation_height(NetworkUpgrade::Nu6_3)
    }
}

impl Parameters for FerryNetwork {
    fn network_type(&self) -> NetworkType {
        match self {
            FerryNetwork::Main => consensus::Network::MainNetwork.network_type(),
            FerryNetwork::Test => consensus::Network::TestNetwork.network_type(),
            FerryNetwork::Regtest(local) => local.network_type(),
        }
    }

    fn activation_height(&self, nu: NetworkUpgrade) -> Option<BlockHeight> {
        match self {
            FerryNetwork::Main => consensus::Network::MainNetwork.activation_height(nu),
            FerryNetwork::Test => consensus::Network::TestNetwork.activation_height(nu),
            FerryNetwork::Regtest(local) => local.activation_height(nu),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regtest_heights_match_heights_env() {
        let net = FerryNetwork::regtest(1, 300);
        assert_eq!(net.network_type(), NetworkType::Regtest);
        assert_eq!(
            net.activation_height(NetworkUpgrade::Nu6_2),
            Some(BlockHeight::from_u32(1))
        );
        assert_eq!(net.ironwood_activation(), Some(BlockHeight::from_u32(300)));
        assert_eq!(net.activation_height(NetworkUpgrade::Nu7), None);
    }

    #[test]
    fn mainnet_has_ironwood() {
        // ZIP 258: NU6.3 activates on mainnet at 3,428,143.
        assert_eq!(
            FerryNetwork::Main.ironwood_activation(),
            Some(BlockHeight::from_u32(3_428_143))
        );
    }
}
