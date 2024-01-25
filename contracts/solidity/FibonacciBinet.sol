// Source: https://medium.com/coinmonks/fibonacci-in-solidity-8477d907e22a

contract FibonacciBinet {
    function fib(uint32 n) public pure returns (uint a) {
        if (n == 0) {
            return 0;
        }
        uint32 h = n / 2;
        uint32 mask = 1;
        // find highest set bit in n
        while (mask <= h) {
            mask <<= 1;
        }
        mask >>= 1;
        a = 1;
        uint b = 1;
        uint c;
        while (mask > 0) {
            c = a * a + b * b;
            if (n & mask > 0) {
                b = b * (b + 2 * a);
                a = c;
            } else {
                a = a * (2 * b - a);
                b = c;
            }
            mask >>= 1;
        }
        return a;
    }
}
