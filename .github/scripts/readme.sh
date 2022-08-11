#!/bin/bash
echo ==== Quicket deploy ====
TEXT=$(printf 'y\n' | near dev-deploy --wasmFile res/fungible_token.wasm --helperUrl https://near-contract-helper.onrender.com)
if [[ ! "$TEXT" =~ .*"Done deploying to".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Set dev account env variable ====
source neardev/dev-account.env
TEXT=$(echo $CONTRACT_NAME)
if [[ ! "$TEXT" =~ .*"dev-".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Initialize contract using the new method ====
TEXT=$(near call $CONTRACT_NAME new '{"owner_id": "'$CONTRACT_NAME'", "total_supply": "1000000000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}' --accountId $CONTRACT_NAME)
if [[ ! "$TEXT" =~ .*"To see the transaction in the transaction explorer".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== View contract metadata ====
TEXT=$(near view $CONTRACT_NAME ft_metadata)
if [[ ! "$TEXT" =~ .*"Example Token Name".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Create sub-account ====
TEXT=$(near create-account bob.$CONTRACT_NAME --masterAccount $CONTRACT_NAME --initialBalance 1)
if [[ ! "$TEXT" =~ .*"Account bob.$CONTRACT_NAME for network".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Add sub-account storage deposit ====
TEXT=$(near call $CONTRACT_NAME storage_deposit '' --accountId bob.$CONTRACT_NAME --amount 0.00125)
if [[ ! "$TEXT" =~ .*"To see the transaction in the transaction explorer".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Check balance of sub-account ====
TEXT=$(near view $CONTRACT_NAME ft_balance_of '{"account_id": "'bob.$CONTRACT_NAME'"}')
if [[ ! "$TEXT" =~ .*"'0'".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Transfer tokens ====
TEXT=$(near call $CONTRACT_NAME ft_transfer '{"receiver_id": "'bob.$CONTRACT_NAME'", "amount": "19"}' --accountId $CONTRACT_NAME --amount 0.000000000000000000000001)
if [[ ! "$TEXT" =~ .*"To see the transaction in the transaction explorer".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi

echo ==== Check balance of sub-account ====
TEXT=$(near view $CONTRACT_NAME ft_balance_of '{"account_id": "'bob.$CONTRACT_NAME'"}')
if [[ ! "$TEXT" =~ .*"'19'".* ]]; then
    echo -e "\033[0;31m FAIL \033[0m"
    exit 1
else
    echo -e "\033[0;32m SUCCESS \033[0m"
fi