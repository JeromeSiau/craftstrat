<?php

use App\Models\Strategy;
use App\Models\Trade;
use App\Models\User;
use App\Models\Wallet;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('returns dashboard stats as Inertia props', function () {
    Strategy::factory()->count(2)->for($this->user)->create(['is_active' => true]);
    Strategy::factory()->for($this->user)->create(['is_active' => false]);
    Wallet::factory()->count(3)->for($this->user)->create();

    $this->actingAs($this->user)
        ->get('/dashboard')
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('dashboard')
            ->has('stats')
            ->where('stats.active_strategies', 2)
            ->where('stats.total_wallets', 3)
            ->where('stats.total_strategies', 3)
        );
});

it('computes dashboard pnl from realized trade outcomes', function () {
    $wallet = Wallet::factory()->for($this->user)->create();
    $strategy = Strategy::factory()->for($this->user)->create();

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'won',
        'price' => 0.400000,
        'filled_price' => 0.400000,
        'resolved_price' => 1.000000,
        'size_usdc' => 10.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'lost',
        'price' => 0.500000,
        'filled_price' => 0.500000,
        'resolved_price' => 0.000000,
        'size_usdc' => 4.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'filled',
        'price' => 0.550000,
        'filled_price' => 0.550000,
        'size_usdc' => 99.000000,
        'executed_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->get('/dashboard')
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->where('stats.total_pnl_usdc', '11.00')
        );
});
