// Compile-fail test for ValidatedBatch::<1500>
// This test verifies that ValidatedBatch::<1500> fails to compile because SIZE = 1500 > MAX_BATCH_SIZE (1000)

use chicago_tdd_tools::guards::validated::ValidatedBatch;

fn main() {
    // This should fail to compile because AssertBatchSize<1500> is not implemented
    let _batch = ValidatedBatch::<1500>::new(vec![0u8; 1500]);
    // Expected compile error: trait bound `(): AssertBatchSize<1500>` is not satisfied
}

