<?php

use App\Models\Strategy;
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
