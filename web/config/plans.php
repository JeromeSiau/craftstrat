<?php

return [

    'free' => [
        'max_wallets' => 1,
        'max_strategies' => 2,
        'max_leaders' => 1,
        'backtest_days' => 30,
    ],

    'starter' => [
        'max_wallets' => 5,
        'max_strategies' => 10,
        'max_leaders' => 5,
        'backtest_days' => null,
    ],

    'pro' => [
        'max_wallets' => 25,
        'max_strategies' => null,
        'max_leaders' => null,
        'backtest_days' => null,
    ],

    'enterprise' => [
        'max_wallets' => null,
        'max_strategies' => null,
        'max_leaders' => null,
        'backtest_days' => null,
    ],

];
