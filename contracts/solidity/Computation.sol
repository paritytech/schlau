contract Computation {
    function triangle_number(int64 n) public pure returns (int64 sum) {
        for (int64 x = 1; x <= n; x++) {
            sum += x;
        }
    }

    function odd_product(int32 n) public pure returns (int64 prod) {
        for (int32 x = 1; x <= n; x++) {
            prod *= 2 * int64(x) - 1;
        }
    }
}
