use anyhow::Result;
use arbitrage::QuoteProvider as StratQuoter;

pub struct MockQuoter;
impl StratQuoter for MockQuoter {
    async fn quote(&self, _i: String, _o: String, amount: u64) -> Result<u64> {
        Ok(amount + 1234) // 随便返回个正收益
    }
}
