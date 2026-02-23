<?php

use App\Models\Strategy;
use App\Models\Trade;
use App\Models\User;
use App\Models\Wallet;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('assigns strategy as paper trading', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.assign-strategy', $wallet), [
            'strategy_id' => $strategy->id,
            'markets' => ['btc-15m'],
            'max_position_usdc' => 200,
            'is_paper' => true,
        ])
        ->assertRedirect();

    $pivot = $wallet->strategies()->first()->pivot;
    expect($pivot->is_paper)->toBeTrue()
        ->and($pivot->max_position_usdc)->toBe('200.000000');
});

it('assigns strategy as live by default', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.assign-strategy', $wallet), [
            'strategy_id' => $strategy->id,
            'markets' => ['btc-15m'],
            'max_position_usdc' => 100,
        ])
        ->assertRedirect();

    $pivot = $wallet->strategies()->first()->pivot;
    expect($pivot->is_paper)->toBeFalse();
});

it('activates paper strategy and passes is_paper to engine', function () {
    Http::fake(['*/internal/strategy/activate' => Http::response(null, 200)]);

    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, [
        'markets' => ['btc-15m'],
        'max_position_usdc' => 200,
        'is_paper' => true,
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.activate', $strategy))
        ->assertRedirect();

    Http::assertSent(function ($request) {
        return $request->url() === config('services.engine.url').'/internal/strategy/activate'
            && $request['is_paper'] === true;
    });
});

it('filters paper trades with scopes', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    Trade::factory()->count(3)->create([
        'strategy_id' => $strategy->id,
        'is_paper' => true,
    ]);
    Trade::factory()->count(2)->create([
        'strategy_id' => $strategy->id,
        'is_paper' => false,
    ]);

    expect($strategy->paperTrades()->count())->toBe(3)
        ->and($strategy->liveTrades()->count())->toBe(2)
        ->and(Trade::paper()->count())->toBe(3)
        ->and(Trade::live()->count())->toBe(2);
});

it('shows separate paper and live stats on strategy page', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->count(2)->create([
        'strategy_id' => $strategy->id,
        'is_paper' => false,
        'status' => 'filled',
        'price' => 0.7,
    ]);
    Trade::factory()->create([
        'strategy_id' => $strategy->id,
        'is_paper' => true,
        'status' => 'filled',
        'price' => 0.6,
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/show', false)
            ->has('strategy')
        );
});
