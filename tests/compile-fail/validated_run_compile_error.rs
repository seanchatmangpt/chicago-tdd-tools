// Compile-fail test for ValidatedRun::<9>
// This test verifies that ValidatedRun::<9> fails to compile because LEN = 9 > MAX_RUN_LEN (8)

use chicago_tdd_tools::guards::validated::ValidatedRun;

fn main() {
    // This should fail to compile because AssertRunLen<9> is not implemented
    let _run = ValidatedRun::<9>::new(vec![0u8; 9]);
    // Expected compile error: trait bound `(): AssertRunLen<9>` is not satisfied
}

