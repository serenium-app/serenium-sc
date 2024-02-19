use gmeta::Metadata;

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = ();
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = (); // Reward logic is stateless, just to save addresses of related contracts
}
