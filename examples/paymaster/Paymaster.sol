// SPDX-License-Identifier: BSD-3-Clause-Clear

pragma solidity ^0.8.20;

import {IPaymaster, ExecutionResult, PAYMASTER_VALIDATION_SUCCESS_MAGIC} from "@matterlabs/zksync-contracts/l2/system-contracts/interfaces/IPaymaster.sol";
import {IPaymasterFlow} from "@matterlabs/zksync-contracts/l2/system-contracts/interfaces/IPaymasterFlow.sol";
import {Transaction} from "@matterlabs/zksync-contracts/l2/system-contracts/libraries/TransactionHelper.sol";
import {BOOTLOADER_FORMAL_ADDRESS} from "@matterlabs/zksync-contracts/l2/system-contracts/Constants.sol";

contract SimplePaymaster is IPaymaster {
    error AccessRestrictedToBootloader();
    error PaymasterFlowNotSupported();
    error NotEnoughETHInPaymasterToPayForTransaction();
    error InvalidPaymasterInput(string message);

    modifier onlyBootloader() {
        if (msg.sender != BOOTLOADER_FORMAL_ADDRESS) {
            revert AccessRestrictedToBootloader();
        }
        _;
    }

    function validateAndPayForPaymasterTransaction(
        bytes32,
        bytes32,
        Transaction calldata transaction
    )
        external
        payable
        onlyBootloader
        returns (bytes4 magic, bytes memory /* context */)
    {
        // By default we consider the transaction as accepted.
        magic = PAYMASTER_VALIDATION_SUCCESS_MAGIC;

        if (transaction.paymasterInput.length < 4) {
            revert InvalidPaymasterInput(
                "The standard paymaster input must be at least 4 bytes long"
            );
        }

        bytes4 paymasterInputSelector = bytes4(transaction.paymasterInput[0:4]);

        // Note, that while the minimal amount of ETH needed is tx.gasPrice * tx.gasLimit,
        // neither paymaster nor account are allowed to access this context variable.
        uint256 requiredETH = transaction.gasLimit * transaction.maxFeePerGas;

        if (paymasterInputSelector != IPaymasterFlow.general.selector) {
            revert PaymasterFlowNotSupported();
        }

        // The bootloader never returns any data, so it can safely be ignored here.
        (bool success, ) = payable(BOOTLOADER_FORMAL_ADDRESS).call{
            value: requiredETH
        }("");
        if (!success) {
            revert NotEnoughETHInPaymasterToPayForTransaction();
        }
    }

    function postTransaction(
        bytes calldata context,
        Transaction calldata transaction,
        bytes32,
        bytes32,
        ExecutionResult txResult,
        uint256 maxRefundedGas
    ) external payable override onlyBootloader {
        // Refunds are not supported
    }

    receive() external payable {}
}
