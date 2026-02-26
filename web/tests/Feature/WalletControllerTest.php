<?php

use App\Jobs\DeploySafeWallet;
use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use App\Services\WalletService;
use Illuminate\Support\Facades\Bus;
use Illuminate\Support\Facades\Http;
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
            ->has('wallets.data', 2)
        );
});

it('creates a new wallet and dispatches deploy job', function () {
    Bus::fake();

    $mock = Mockery::mock(WalletService::class);
    $mock->shouldReceive('generateKeypair')->once()->andReturn([
        'signer_address' => '0xAbCdEf1234567890AbCdEf1234567890AbCdEf12',
        'private_key_enc' => base64_encode('encrypted-key'),
    ]);
    $this->app->instance(WalletService::class, $mock);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'My Wallet'])
        ->assertRedirect();

    $wallet = Wallet::where('user_id', $this->user->id)->first();

    expect($wallet)->not->toBeNull()
        ->and($wallet->label)->toBe('My Wallet')
        ->and($wallet->signer_address)->toBe('0xAbCdEf1234567890AbCdEf1234567890AbCdEf12')
        ->and($wallet->status)->toBe('pending')
        ->and($wallet->safe_address)->toBeNull();

    Bus::assertDispatched(DeploySafeWallet::class, fn ($job) => $job->wallet->id === $wallet->id);
});

it('deletes a deployed wallet belonging to the user', function () {
    Http::fake(['*/internal/strategy/deactivate' => Http::response(null, 200)]);

    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertRedirect(route('wallets.index'));

    expect(Wallet::find($wallet->id))->toBeNull();
});

it('prevents deleting a wallet while deploying', function () {
    $wallet = Wallet::factory()->deploying()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertRedirect()
        ->assertSessionHas('error');

    expect(Wallet::find($wallet->id))->not->toBeNull();
});

it('prevents deleting another users wallet', function () {
    $other = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertForbidden();
});

it('retries a failed wallet deployment', function () {
    Bus::fake();

    $wallet = Wallet::factory()->failed()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.retry', $wallet))
        ->assertRedirect();

    expect($wallet->fresh()->status)->toBe('pending');
    Bus::assertDispatched(DeploySafeWallet::class, fn ($job) => $job->wallet->id === $wallet->id);
});

it('prevents retrying a non-failed wallet', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id, 'status' => 'deployed']);

    $this->actingAs($this->user)
        ->post(route('wallets.retry', $wallet))
        ->assertRedirect()
        ->assertSessionHas('error');
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

it('wallet index includes available strategies', function () {
    Strategy::factory()->for($this->user)->create(['name' => 'Test Strategy']);

    $this->actingAs($this->user)
        ->get(route('wallets.index'))
        ->assertInertia(fn (Assert $page) => $page
            ->component('wallets/index', false)
            ->has('strategies', 1)
        );
});

it('requires authentication for wallets', function () {
    $this->get(route('wallets.index'))->assertRedirect('/login');
});
