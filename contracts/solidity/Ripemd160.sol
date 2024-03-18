contract Ripemd160 {
    function rmd160(bytes memory preimage) public pure returns (bytes20) {
        return ripemd160(preimage);
    }
}
