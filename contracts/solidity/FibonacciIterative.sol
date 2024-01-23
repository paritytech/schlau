contract FibonacciIterative {
    function fib(uint32 n) public pure returns (uint b) {
        if (n == 0) {
            return 0;
        }

        uint a = 1;
        b = 1;
        for (uint32 i = 2; i < n; i++) {
            uint c = a + b;
            a = b;
            b = c;
        }
        return b;
    }
}
