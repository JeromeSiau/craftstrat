<?php

use App\Models\User;
use App\Models\Wallet;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('allows owner to view their wallet', function () {
    $user = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $user->id]);

    expect($user->can('view', $wallet))->toBeTrue();
});

it('prevents user from viewing another users wallet', function () {
    $owner = User::factory()->create();
    $other = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $owner->id]);

    expect($other->can('view', $wallet))->toBeFalse();
});

it('allows owner to delete their wallet', function () {
    $user = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $user->id]);

    expect($user->can('delete', $wallet))->toBeTrue();
});
