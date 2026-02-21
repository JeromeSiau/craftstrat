<?php

use App\Models\Strategy;
use App\Models\User;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('allows owner to view their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('view', $strategy))->toBeTrue();
});

it('prevents user from viewing another users strategy', function () {
    $owner = User::factory()->create();
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $owner->id]);

    expect($other->can('view', $strategy))->toBeFalse();
});

it('allows owner to update their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('update', $strategy))->toBeTrue();
});

it('allows owner to delete their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('delete', $strategy))->toBeTrue();
});
