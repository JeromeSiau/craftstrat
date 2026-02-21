<?php

use App\Models\BacktestResult;
use App\Models\CopyRelationship;
use App\Models\CopyTrade;
use App\Models\Strategy;
use App\Models\Trade;
use App\Models\User;
use App\Models\Wallet;
use App\Models\WatchedWallet;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('creates a trade belonging to a wallet and strategy', function () {
    $trade = Trade::factory()->create();

    expect($trade->wallet)->toBeInstanceOf(Wallet::class)
        ->and($trade->strategy)->toBeInstanceOf(Strategy::class);
});

it('creates a watched wallet with relationships', function () {
    $watched = WatchedWallet::factory()->create();
    $relationship = CopyRelationship::factory()->create(['watched_wallet_id' => $watched->id]);

    expect($watched->copyRelationships)->toHaveCount(1);
});

it('creates a copy relationship linking follower to leader', function () {
    $relationship = CopyRelationship::factory()->create();

    expect($relationship->followerWallet)->toBeInstanceOf(Wallet::class)
        ->and($relationship->watchedWallet)->toBeInstanceOf(WatchedWallet::class);
});

it('creates a copy trade linked to a relationship', function () {
    $copyTrade = CopyTrade::factory()->create();

    expect($copyTrade->copyRelationship)->toBeInstanceOf(CopyRelationship::class);
});

it('creates a backtest result belonging to user and strategy', function () {
    $result = BacktestResult::factory()->create();

    expect($result->user)->toBeInstanceOf(User::class)
        ->and($result->strategy)->toBeInstanceOf(Strategy::class)
        ->and($result->result_detail)->toBeNull();
});

it('casts backtest result fields correctly', function () {
    $result = BacktestResult::factory()->create([
        'total_trades' => 42,
        'win_rate' => 0.6523,
        'result_detail' => ['trades' => []],
    ]);

    expect($result->total_trades)->toBe(42)
        ->and($result->win_rate)->toBe('0.6523')
        ->and($result->result_detail)->toBeArray();
});
