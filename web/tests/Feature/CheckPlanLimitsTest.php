<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use App\Services\WalletService;

beforeEach(function () {
    $this->user = User::factory()->create(['plan' => 'free']);

    $mock = Mockery::mock(WalletService::class);
    $mock->shouldReceive('generateKeypair')->andReturn([
        'address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
        'private_key_enc' => base64_encode('encrypted'),
    ]);
    $this->app->instance(WalletService::class, $mock);
});

it('allows creating a wallet when under limit', function () {
    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 1'])
        ->assertRedirect()
        ->assertSessionMissing('error');

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1);
});

it('blocks creating a wallet when at limit', function () {
    Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 2'])
        ->assertRedirect()
        ->assertSessionHas('error');

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1);
});

it('allows pro plan users to create many wallets', function () {
    $this->user->update(['plan' => 'pro']);
    Wallet::factory()->count(10)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 11'])
        ->assertRedirect()
        ->assertSessionMissing('error');
});

it('blocks creating strategies when at limit for free plan', function () {
    Strategy::factory()->count(2)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'Blocked',
            'mode' => 'form',
            'graph' => ['mode' => 'form', 'conditions' => [], 'action' => [], 'risk' => []],
        ])
        ->assertRedirect()
        ->assertSessionHas('error');
});

it('allows creating strategies when under limit', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'Allowed',
            'mode' => 'form',
            'graph' => ['mode' => 'form', 'conditions' => [], 'action' => [], 'risk' => []],
        ])
        ->assertRedirect()
        ->assertSessionMissing('error');

    expect(Strategy::where('user_id', $this->user->id)->count())->toBe(1);
});
