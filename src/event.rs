use ethers::types::Address;

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Disconnected,
    AccountsChanged(Option<Vec<Address>>),
    ChainIdChanged(u64),
    Broken,
}
