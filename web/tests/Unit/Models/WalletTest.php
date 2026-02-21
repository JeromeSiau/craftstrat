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
