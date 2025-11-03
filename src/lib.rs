pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
pub mod helper;
pub mod instructions;
pub mod state;

pub use helper::*;
pub use state::*;

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
