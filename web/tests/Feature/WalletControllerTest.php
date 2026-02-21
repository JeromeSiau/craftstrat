<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use App\Services\WalletService;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays wallets index page', function () {
    Wallet::factory()->count(2)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('wallets.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('wallets/index', false)
            ->has('wallets', 2)
        );
});

it('creates a new wallet with generated keypair', function () {
    $mock = Mockery::mock(WalletService::class);
    $mock->shouldReceive('generateKeypair')->once()->andReturn([
        'address' => '0xAbCdEf1234567890AbCdEf1234567890AbCdEf12',
        'private_key_enc' => base64_encode('encrypted-key'),
    ]);
    $this->app->instance(WalletService::class, $mock);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'My Wallet'])
        ->assertRedirect();

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1)
        ->and(Wallet::first()->label)->toBe('My Wallet')
        ->and(Wallet::first()->address)->toBe('0xAbCdEf1234567890AbCdEf1234567890AbCdEf12');
});

it('deletes a wallet belonging to the user', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertRedirect(route('wallets.index'));

    expect(Wallet::find($wallet->id))->toBeNull();
});

it('prevents deleting another users wallet', function () {
    $other = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertForbidden();
});

it('assigns a strategy to a wallet', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.assign-strategy', $wallet), [
            'strategy_id' => $strategy->id,
            'markets' => ['btc-15m'],
            'max_position_usdc' => 200,
        ])
        ->assertRedirect();

    expect($wallet->strategies()->count())->toBe(1);
});

it('removes a strategy from a wallet', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, ['markets' => [], 'max_position_usdc' => 100]);

    $this->actingAs($this->user)
        ->delete(route('wallets.remove-strategy', [$wallet, $strategy]))
        ->assertRedirect();

    expect($wallet->strategies()->count())->toBe(0);
});

it('requires authentication for wallets', function () {
    $this->get(route('wallets.index'))->assertRedirect('/login');
});
