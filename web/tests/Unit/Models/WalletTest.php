<?php

use App\Models\User;
use App\Models\Wallet;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('belongs to a user', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->user)->toBeInstanceOf(User::class);
});

it('hides private_key_enc from serialization', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->toArray())->not->toHaveKey('private_key_enc');
});

it('casts balance_usdc as decimal', function () {
    $wallet = Wallet::factory()->create(['balance_usdc' => 100.123456]);

    expect($wallet->balance_usdc)->toBe('100.123456');
});

it('casts is_active as boolean', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->is_active)->toBeTrue();
});

it('reports correct deployment status', function () {
    $deployed = Wallet::factory()->create();
    $pending = Wallet::factory()->pending()->create();
    $deploying = Wallet::factory()->deploying()->create();
    $failed = Wallet::factory()->failed()->create();

    expect($deployed->isDeployed())->toBeTrue()
        ->and($deployed->isDeploying())->toBeFalse()
        ->and($deployed->isFailed())->toBeFalse()
        ->and($pending->isDeployed())->toBeFalse()
        ->and($pending->isDeploying())->toBeTrue()
        ->and($deploying->isDeployed())->toBeFalse()
        ->and($deploying->isDeploying())->toBeTrue()
        ->and($failed->isDeployed())->toBeFalse()
        ->and($failed->isFailed())->toBeTrue();
});

it('casts deployed_at as datetime', function () {
    $wallet = Wallet::factory()->create(['deployed_at' => '2026-01-15 12:00:00']);

    expect($wallet->deployed_at)->toBeInstanceOf(\Carbon\CarbonImmutable::class);
});
