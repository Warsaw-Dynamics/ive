// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "./IERC20.sol";

/* Orders are paid in tokens and refunded within a week. */
interface IVault {
    function deposit(uint256 amount) external returns (bool);
}

/// @title Order book of one shop
/// @notice Keeps paid orders until they are delivered.
contract OrderBook is IVault {
    enum Status { Open, Paid, Delivered }

    struct Order {
        address buyer;
        uint128 price;
        Status status;
    }

    uint256 public constant MAX_ORDERS = 1_000;
    uint256 private immutable refundWindow = 7 days;
    mapping(uint256 id => Order) public orders;
    IERC20 public token;

    event Paid(address indexed buyer, uint256 id);
    error TooMany(uint256 count);

    modifier onlyBuyer(uint256 id) {
        require(orders[id].buyer == msg.sender, "not the buyer");
        _;
    }

    function deposit(uint256 amount) external override returns (bool) {
        checkLimit(amount);
        Order memory order = Order({buyer: msg.sender, price: uint128(amount), status: Status.Paid});
        orders[amount] = order;
        emit Paid(msg.sender, amount);
        return token.transferFrom(msg.sender, address(this), amount);
    }

    function checkLimit(uint256 amount) private pure {
        if (amount > MAX_ORDERS) revert TooMany(amount);
    }

    function size() public view onlyBuyer(0) returns (uint256 total) {
        assembly {
            let slot := orders.slot
            total := sload(slot)
        }
    }
}
