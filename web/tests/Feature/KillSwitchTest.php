<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use Illuminate\Support\Facades\Http;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('sends kill request to engine for all wallet assignments', function () {
    Http::fake(['*/internal/strategy/kill' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, ['is_running' => true, 'max_position_usdc' => 200]);

    $this->actingAs($this->user)
        ->post(route('strategies.kill', $strategy))
        ->assertRedirect()
        ->assertSessionHas('success');

    Http::assertSent(function ($request) use ($wallet, $strategy) {
        return str_contains($request->url(), '/internal/strategy/kill')
            && $request['wallet_id'] === $wallet->id
            && $request['strategy_id'] === $strategy->id;
    });
});

it('sends unkill request to engine for all wallet assignments', function () {
    Http::fake(['*/internal/strategy/unkill' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, ['is_running' => true, 'max_position_usdc' => 200]);

    $this->actingAs($this->user)
        ->post(route('strategies.unkill', $strategy))
        ->assertRedirect()
        ->assertSessionHas('success');

    Http::assertSent(function ($request) use ($wallet, $strategy) {
        return str_contains($request->url(), '/internal/strategy/unkill')
            && $request['wallet_id'] === $wallet->id
            && $request['strategy_id'] === $strategy->id;
    });
});

it('prevents unauthorized users from killing strategy', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->active()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.kill', $strategy))
        ->assertForbidden();
});

it('returns error when engine is unavailable for kill', function () {
    Http::fake(['*/internal/strategy/kill' => Http::response(null, 500)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, ['is_running' => true, 'max_position_usdc' => 200]);

    $this->actingAs($this->user)
        ->post(route('strategies.kill', $strategy))
        ->assertRedirect()
        ->assertSessionHas('error');
});

it('requires authentication for kill switch', function () {
    $strategy = Strategy::factory()->create();

    $this->post(route('strategies.kill', $strategy))
        ->assertRedirect('/login');
});
