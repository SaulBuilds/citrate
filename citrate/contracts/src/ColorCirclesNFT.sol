// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Base64.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

/**
 * @title ColorCirclesNFT
 * @dev On-chain SVG NFT with 256 colorful circles
 */
contract ColorCirclesNFT is ERC721, Ownable {
    using Strings for uint256;

    uint256 public constant MAX_SUPPLY = 256;
    uint256 private _currentTokenId = 0;

    constructor() ERC721("Color Circles", "CIRCLES") Ownable(msg.sender) {}

    /**
     * @dev Mint a new NFT
     */
    function mint() external returns (uint256) {
        require(_currentTokenId < MAX_SUPPLY, "Max supply reached");

        uint256 tokenId = _currentTokenId;
        _currentTokenId++;

        _safeMint(msg.sender, tokenId);

        return tokenId;
    }

    /**
     * @dev Batch mint multiple NFTs
     */
    function mintBatch(uint256 amount) external {
        require(_currentTokenId + amount <= MAX_SUPPLY, "Exceeds max supply");

        for (uint256 i = 0; i < amount; i++) {
            uint256 tokenId = _currentTokenId;
            _currentTokenId++;
            _safeMint(msg.sender, tokenId);
        }
    }

    /**
     * @dev Get the total supply
     */
    function totalSupply() external view returns (uint256) {
        return _currentTokenId;
    }

    /**
     * @dev Generate color based on token ID
     */
    function getColor(uint256 tokenId) public pure returns (string memory) {
        require(tokenId < MAX_SUPPLY, "Invalid token ID");

        // Use token ID to generate RGB values
        uint256 r = (tokenId * 137 + 50) % 256;
        uint256 g = (tokenId * 197 + 100) % 256;
        uint256 b = (tokenId * 233 + 150) % 256;

        return string(abi.encodePacked(
            "rgb(",
            r.toString(),
            ",",
            g.toString(),
            ",",
            b.toString(),
            ")"
        ));
    }

    /**
     * @dev Generate SVG for a token
     */
    function generateSVG(uint256 tokenId) public pure returns (string memory) {
        require(tokenId < MAX_SUPPLY, "Invalid token ID");

        string memory color = getColor(tokenId);

        return string(abi.encodePacked(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" width="400" height="400">',
            '<rect width="400" height="400" fill="#1a1a1a"/>',
            '<circle cx="200" cy="200" r="150" fill="',
            color,
            '" opacity="0.9"/>',
            '<circle cx="200" cy="200" r="120" fill="',
            color,
            '" opacity="0.7"/>',
            '<circle cx="200" cy="200" r="90" fill="',
            color,
            '" opacity="0.5"/>',
            '<circle cx="200" cy="200" r="60" fill="',
            color,
            '" opacity="0.3"/>',
            '<text x="200" y="380" font-family="monospace" font-size="24" fill="white" text-anchor="middle">',
            '#',
            tokenId.toString(),
            '</text>',
            '</svg>'
        ));
    }

    /**
     * @dev Generate metadata JSON for a token
     */
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(ownerOf(tokenId) != address(0), "Token does not exist");

        string memory svg = generateSVG(tokenId);
        string memory color = getColor(tokenId);

        string memory json = string(abi.encodePacked(
            '{"name":"Color Circle #',
            tokenId.toString(),
            '","description":"An on-chain generative SVG circle with unique colors from the 256-piece collection.","image":"data:image/svg+xml;base64,',
            Base64.encode(bytes(svg)),
            '","attributes":[',
            '{"trait_type":"Color","value":"',
            color,
            '"},',
            '{"trait_type":"Token ID","value":"',
            tokenId.toString(),
            '"},',
            '{"trait_type":"Series","value":"Color Circles"},',
            '{"trait_type":"Supply","value":"256"}',
            ']}'
        ));

        return string(abi.encodePacked(
            "data:application/json;base64,",
            Base64.encode(bytes(json))
        ));
    }
}
