contract Arithmetics {
    function remainders(
        uint256 xL_in,
        uint256 xR_in
    ) external pure returns (uint256, uint256) {
        uint q = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;

        for (uint i = 0; i < 256; i++) {
            xL_in = mulmod(xL_in, xR_in, q);
            xR_in = addmod(xL_in, xR_in, q);
        }

        return (xL_in, xR_in);
    }
}
